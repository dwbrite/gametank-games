import FontTest from "./debug"
import RustEmu from "./emu"

alignToPixelGrid = (selector) ->
  elements = document.querySelectorAll(selector) # Get all matching elements
  for element in elements
    # Temporarily remove any inline transform
    inlineTransform = element.style.transform
    element.style.transform = ''

    # Get the computed transform and bounding box
    computedTransform = window.getComputedStyle(element).getPropertyValue('transform') or 'none'
    rect = element.getBoundingClientRect()
    offset = Math.round(rect.left) - rect.left

    # Reapply the computed transform with the pixel adjustment
    if computedTransform == 'none'
      element.style.transform = "translateX(#{-offset}px)"
    else
      element.style.transform = "#{computedTransform} translateX(#{-offset}px)"

    # Restore the original inline transform (if necessary)
    if inlineTransform
      element.style.transform = "#{element.style.transform}"

GameEntry =
  view: (vnode) ->
    game = vnode.attrs.game

    <li className="game-entry pixels">
      <img className="thumbnail" src="https://i.redd.it/pvegyycnjkb71.jpg"></img>

      <div>
        <m.route.Link href={"/game/" + game.metadata.game_id}>
          <h3>{game.metadata.game_name}</h3>
        </m.route.Link>
        <m.route.Link className="author_name" href={"/user/" + game.metadata.author}>
          {game.author_name}, {game.metadata.created_at.split('T')[0]}
        </m.route.Link>
        <hr/>
        <p>{game.metadata.description}</p>

      </div>
    </li>

GameList =
  games: []       # Initialize an empty array to hold game data
  loading: true   # Loading state
  error: null     # Error state

  oninit: ->
    # Fetch the games when the component initializes
    Api.list_games()
      .then (data) ->
        GameList.games = data
        GameList.loading = false
      .catch (err) ->
        GameList.error = "Failed to load games."
        GameList.loading = false

  view: ->
    <div className="game-list-container">{
      if GameList.loading
        <div className="loading">Loading games...</div>
      else if GameList.error?
        <div className="error">{GameList.error}</div>
      else if GameList.games.length == 0
        <div>No games available.</div>
      else
        <ul className="game-list">
          {GameList.games.map (game) ->
            <GameEntry game={game}/>
          }
        </ul>
    }</div>

UserMenu =
  state:
    initialized: false
    user_info: null
    # Static state map
    state_map:
      initializing:
        welcome_text: () =>
          <span className="welcome-text">
            Initializing...
          </span>
        buttons: ["scale_up", "scale_down"]
      not_authenticated:
        welcome_text: () =>
          <span className="welcome-text">
            Welcome, <span className="username">Guest</span>
          </span>
        buttons: ["login", "scale_up", "scale_down"]
      authenticated:
        welcome_text: (user_info) =>
          <span className="welcome-text">
            Welcome, <span className="username">
              {user_info?.preferred_username or "..."}
            </span>
          </span>
        buttons: ["profile", "upload", "logout", "scale_up", "scale_down"]
    # Static button map
    button_map:
      login:
        label: <span>Login</span>
        action: Api.login
      profile:
        label: <span>Profile</span>
        action: -> m.route.set "/profile"
      upload:
        label: <span>Upload</span>
        action: ->  m.route.set "/upload"
      logout:
        label: <span>Logout</span>
        action: Api.logout
      scale_up:
        label: <code>+</code>
        action: ->
          currentScale = parseFloat(getComputedStyle(document.body).getPropertyValue("--scale")) or 1
          newScale = Math.min(3, currentScale + 1)
          document.body.style.setProperty "--scale", newScale
          localStorage.setItem "scale", newScale
      scale_down:
        label: <code>-</code>
        action: ->
          currentScale = parseFloat(getComputedStyle(document.body).getPropertyValue("--scale")) or 1
          newScale = Math.max(1, currentScale - 1) # Prevent scale < 1
          document.body.style.setProperty "--scale", newScale
          localStorage.setItem "scale", newScale

  oninit: ->
    Api.load_user_info().then (data) =>
      this.state.user_info = data

  view: ->
    current_state =
      if not Api.initialized then "initializing"
      else if not Api.authenticated() then "not_authenticated"
      else "authenticated"

    state_config = this.state.state_map[current_state]

    welcome_text = state_config.welcome_text(this.state.user_info)

    <div className="user-menu">
      <span className="welcome-text">{welcome_text}</span>
      <div className="user-menu-buttons">
        {
          state_config.buttons.map (key) =>
            button = this.state.button_map[key]
            <button onclick={button.action}>{button.label}</button>
        }
      </div>
    </div>


import Api from './api'
import Upload from './upload'

import init, {update_rom_data, request_close} from './bin/gametank-emu-rs'


ShadowCanvas =
  oncreate: (vnode) ->
    host = vnode.dom
    # Create a shadow root in "open" mode so you can access it from JS
    shadow = host.attachShadow({mode: "open"})
    host.style.visibility = "hidden"  # Hide the host initially
    host.style.position = "absolute"
    canvas = document.createElement("canvas")
    canvas.id = "gt-canvas"
    shadow.appendChild(canvas)
    requestAnimationFrame -> init()

  view: ->
    <div id="shadow-host"></div>

Site =
  oninit: ->
    savedScale = parseFloat(localStorage.getItem("scale")) or 2
    document.body.style.setProperty "--scale", savedScale

    savedScale = parseFloat(localStorage.getItem("emu-scale")) or 3
    document.body.style.setProperty "--emu-scale", savedScale

    window.onresize = -> alignToPixelGrid ".pixels, div, span"


  onupdate: -> alignToPixelGrid ".pixels, div"

  view: (vnode) ->
    <div className="the-page pixels">
      <nav className="navigation">
        <m.route.Link className="nav-title" href={"/"}>
          <h1>GAMETANK.GAMES</h1>
        </m.route.Link>
        <UserMenu/>
        <SisterLinks/>
      </nav>
      <ShadowCanvas/>
      <div className="the-content">
        { vnode.children }
      </div>
    </div>

SisterLinks =
  view: (vnode) ->
    <div>
      <a href="https://gametank.zone">GameTank Zone</a>
    </div>

Profile =
  view: ->
    <div><a href="/#!/fonts">test fonts</a></div>
    <div><a href="/#!/emu">test emu</a></div>

Api.init()
  .then ->
    m.route document.body, "/",
      "/": render: -> <Site><GameList/></Site>
      "/upload": render: -> <Site><Upload/></Site>
      "/profile": render: -> <Site><Profile/></Site>
      "/fonts": render: -> <Site><FontTest/></Site>
      "/game/:game_id": render: ({attrs}) -> <Site><RustEmu gameId={attrs.game_id}/></Site>
  .catch ->
    console.log "failed to start api :)"