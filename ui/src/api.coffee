import Keycloak from 'keycloak-js'

keycloak = new Keycloak({
  url: 'https://keycloak.dwbrite.com',
  realm: 'gametank-games',
  clientId: 'login-frontend',
})

Api =
  initialized: false
  init_started: false
  init_promise: null

  init: ->
    return Promise.resolve() if Api.initialized

    # If there’s a pending init, return it.
    return Api.init_promise if Api.init_promise?

    Api.init_started = true
    console.log 'Initializing Keycloak...'

    Api.init_promise = (() =>
      try
        await keycloak.init onLoad: 'check-sso', responseMode: 'query'
        Api.initialized = true
        console.log 'Keycloak initialized'
      catch err
        console.error 'Error initializing Keycloak:', err
        Api.initialized = false
        Api.init_started = false
        Api.init_promise = null
        throw err
    )()

    Api.init_promise

  login: -> keycloak.login()
  logout: -> keycloak.logout()
  authenticated: -> keycloak.authenticated

  # Authenticated API Methods (shimmed later)
  load_user_info: ->
    m.request
      url: '/api/user-info',
      method: 'GET'
      headers:
        'Authorization': 'Bearer ' + keycloak.token
        'Content-Type': 'application/json'

  create_game: (game_name, description, game_rom) ->
    m.request
      url: '/api/games',
      method: 'POST'
      headers:
        'Authorization': 'Bearer ' + keycloak.token
        'Content-Type': 'application/json'
      body: JSON.stringify
        game_name: game_name or 'Untitled Game'
        description: description or 'No description provided.'
        game_rom: game_rom or []

  list_games: ->
    m.request
      url: '/api/games',
      method: 'GET'
      headers:
        'Authorization': 'Bearer ' + keycloak.token
        'Content-Type': 'application/json'

# Shim for token refresh
shimApiWithTokenRefresh = (api, keycloak) ->
  shimmedApi = {}
  Object.keys(api).forEach (key) ->
    originalFn = api[key]

    # Skip lifecycle methods
    if key in ['init', 'login', 'logout', 'authenticated']
      shimmedApi[key] = originalFn
    else
      shimmedApi[key] = (...args) ->
        await Api.init()
        if keycloak.authenticated
          try
            await keycloak.updateToken(30)
          catch err
            console.error 'Failed to refresh token', err
            throw new Error 'Token refresh failed'

        originalFn(...args)

  shimmedApi

export default shimApiWithTokenRefresh(Api, keycloak)
