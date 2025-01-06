keycloak = new Keycloak({
  url: 'https://keycloak.dwbrite.com',
  realm: 'gametank-games',
  clientId: 'login-frontend',
})

try
  authenticated = await keycloak.init(
    onLoad: 'check-sso',
  )
  if (authenticated)
    console.log('User is authenticated')
  else
    console.log('User is not authenticated')
catch error
  console.error('Failed to initialize adapter:', error)

Api =
  load_user_info: ->
    console.log('Fetching user info...')
    fetch('/api/user-info', {
      method: 'GET',
      headers: {
        'Authorization': 'Bearer ' + keycloak.token,  # Send Keycloak token
        'Content-Type': 'application/json'
      }
    })
      .then((response) ->
        if response.ok
          return response.json();
        else
          throw new Error("Failed to fetch user info")
    )
      .then((data) ->
        console.log('User info from backend:', data)
    )
      .catch((error) ->
        console.error('Error fetching user info:', error)
    )

  create_resource: ->
    console.log("creating resource")
    fetch('/api/games', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + keycloak.token, # Send Keycloak token
        'Content-Type': 'application/json'
      },
      body: JSON.stringify {
        "game_name": "My First Game",
        "description": "This is a cool game with no ROM data yet!",
        "game_rom": []
      }

    })

  list_games: ->
    console.log("listing games")
    fetch("/api/games", {
      method: 'GET',
      headers: {
        'Authorization': 'Bearer ' + keycloak.token, # Send Keycloak token
        'Content-Type': 'application/json'
      }
    }).then (response) ->
      response.json().then (data) ->
        return data

shimApiWithTokenRefresh = (api) ->
  shimmedApi = {}

  Object.keys(api).forEach (key) ->
    originalFn = api[key]
    shimmedApi[key] = (...args) ->
      if authenticated
        try
          await keycloak.updateToken(30)
        catch err
          console.error 'Failed to refresh token', err
          throw new Error 'Token refresh failed'

      originalFn(...args)

  shimmedApi

GameEntry =
  view: (vnode) ->
    game = vnode.attrs.game

    <div className="game-entry">
      <h2>{game.game_name}</h2>
      <p>{game.description}</p>
      <p><strong>Author:</strong> {game.author}</p>
      <p><strong>Created At:</strong> {new Date(game.created_at).toLocaleString()}</p>
      <p><strong>Updated At:</strong> {new Date(game.updated_at).toLocaleString()}</p>
    </div>

GameList =
  games: []       # Initialize an empty array to hold game data
  loading: true   # Loading state
  error: null     # Error state

  oninit: ->
# Fetch the games when the component initializes
    Auth.api.list_games()
      .then (data) ->
        GameList.games = data
        GameList.loading = false
        m.redraw()# Trigger a redraw to update the view
      .catch (err) ->
        GameList.error = "Failed to load games."
        GameList.loading = false
        m.redraw()

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

Auth =
  api: shimApiWithTokenRefresh(Api)

  login: ->
    console.log("logging in???")
    authenticated = keycloak.login()

  logout: ->
    await keycloak.logout()
    console.log('User logged out')

  view: ->
    <div className="auth-container">
      <div className="auth-buttons">
        <button onclick={Auth.login} disabled={Auth.authenticated}>Login</button>
        <button onclick={Auth.logout} disabled={!Auth.authenticated}>Logout</button>
        <button onclick={Auth.api.load_user_info}>User Info</button>
        <button onclick={Auth.api.create_resource}>Create Game</button>
      </div>


      <div className="game-section">
        <h2>Game List</h2>
        <GameList/>
      </div>
    </div>


m.route(document.body, "/", {
  "/": Auth
})
