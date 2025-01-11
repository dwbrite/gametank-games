import FontTest from "./debug"

alignToPixelGrid = (selector) ->
  elements = document.querySelectorAll(selector) # Get all matching elements
  for element in elements
    rect = element.getBoundingClientRect()
    offset = Math.round(rect.left) - rect.left
    element.style.setProperty 'transform', "translateX(#{-offset}px)"

window.addEventListener 'resize', -> alignToPixelGrid(".the-page")


GameEntry =
  view: (vnode) ->
    game = vnode.attrs.game

    <div className="game-entry pixels">
      <img src="https://i.redd.it/pvegyycnjkb71.jpg" width="256px" height="256px"></img>

      <div>
        <h3>{game.metadata.game_name}</h3>
        <a className="author_name" href={"/users/" + game.metadata.author}>{game.author_name}</a>

        <p>{game.metadata.description}</p>

        <p>Created At: {new Date(game.metadata.created_at).toLocaleString()}</p>
      </div>
    </div>

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
    if GameList.loading
      <div className="loading">Loading games...</div>
    else if GameList.error?
      <div className="error">{GameList.error}</div>
    else if GameList.games.length == 0
      <div>No games available.</div>
    else
      <div className="game-list">
        {GameList.games.map (game) ->
          <GameEntry game={game}/>
        }
      </div>

UserMenu =
  state:
    initialized: false
    user_info: null

  oninit: ->
    Api.load_user_info().then (data) =>
      this.state.user_info = data

  view: ->
    if not Api.initialized
      <div className="user-menu">
        Initializing...
      </div>
    else if not Api.authenticated()
      <div className="user-menu">
        <span className="username">Guest</span>
        <div className="user-menu-buttons">
          <button onclick={Api.login}>Login</button>
        </div>
      </div>
    else
      <div className="user-menu">
        <span className="welcome-text">
          Welcome, <span className="username">
            { this.state.user_info?.preferred_username or "..."}
          </span>
        </span>
        <div className="user-menu-buttons">
          <button onclick={-> m.route.set "/profile"}>Profile</button>
          <button onclick={Api.logout}>Logout</button>
        </div>
      </div>

import Api from './api'

Site =
  onupdate: -> alignToPixelGrid ".pixels"

  view: (vnode) ->
    <div className="the-page pixels">
      <nav className="navigation">
        <m.route.Link className="nav-title" href={"/"}>
          <h1>GAMETANK.GAMES</h1>
        </m.route.Link>
        <UserMenu/>
      </nav>
      <div className="the-content">
        { vnode.children }
      </div>
    </div>

Profile =
  view: ->
    <div><a href="/#!/fonts">test fonts</a></div>

Api.init()
  .then ->
    m.route document.body, "/",
      "/": render: -> <Site><GameList/></Site>
      "/profile": render: -> <Site><Profile/></Site>
      "/fonts": render: -> <Site><FontTest/></Site>
  .catch ->
    console.log "failed to start api :)"