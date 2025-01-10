require 'coffeescript/register'


GameEntry =
  view: (vnode) ->
    game = vnode.attrs.game

    <div className="game-entry">
      <img src="https://i.redd.it/pvegyycnjkb71.jpg" width="128px" height="128px"></img>

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
    AuthenticatedApi.api.list_games()
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

#
#AuthenticatedApi =
#
#  view: ->
#    <div className="auth-container">
#      <h1>gametank.gamess</h1>
#      <div className="auth-buttons">
#        <button onclick={this.login} disabled={keycloak.authenticated}>Login</button>
#        <button onclick={this.logout} disabled={!keycloak.authenticated}>Logout</button>
#        <button onclick={this.api.load_user_info}>User Info</button>
#        <button onclick={this.api.create_resource}>Create Game</button>
#      </div>
#
#
#      <div className="game-section">
#        <h2>Game List</h2>
#        <GameList/>
#      </div>
#    </div>

UserMenu =
  state: {
    initialized: false
    user_info: null
  }

  oninit: ->
    Api.load_user_info().then (data) =>
      this.state.user_info = data

  view: (vnode) ->
    if not Api.initialized
      <div className="userMenu">
        Initializing...
      </div>
    else if not Api.authenticated()
      <div className="userMenu">
        <span>Guest</span>
        <a href="#" onclick={(e) =>
          e.preventDefault()
          Api.login()
        }>Login</a>
      </div>
    else
      <div className="userMenu">
        <span>Welcome, { this.state.user_info?.preferred_username or "..."}</span>
        <a href="#" onclick={(e) =>
          e.preventDefault()
          console.log("logging out")
          Api.logout()
        }>Logout</a>
      </div>

import Api from './api'

Site =
  view: ->
    <div className="the-page">
      <div className="the-top">
        <h1>GAMETANK.GAMES</h1>
        <UserMenu/>
      </div>
      <div className="the-games">
          hi
      </div>
    </div>

Profile =
  view: ->
    <div><a href="/">go back</a></div>

m.route.prefix = ""
m.route(document.body, "/", {
  "/": Site,
  "/profile": Profile
})

