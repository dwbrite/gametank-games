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
    fetch('/api/create-resource', {
      method: 'POST',
      headers: {
        'Authorization': 'Bearer ' + keycloak.token,  # Send Keycloak token
        'Content-Type': 'application/json'
      }  # Convert resource object to JSON
    })

shimApiWithTokenRefresh = (api) ->
  shimmedApi = {}

  Object.keys(api).forEach (key) ->
    originalFn = api[key]
    shimmedApi[key] = async (...args) ->
      try
        await keycloak.updateToken(30)
      catch err
        console.error 'Failed to refresh token', err
        throw new Error 'Token refresh failed'

      originalFn(...args)

  shimmedApi

Auth =
  api: shimApiWithTokenRefresh(Api)
  login: ->
    console.log("logging in???")
    authenticated = keycloak.login()

  logout: ->
    await keycloak.logout()
    console.log('User logged out')

  view: ->
    <div>
      <button onclick={Auth.login} disabled={authenticated}>Login</button>
      <button onclick={Auth.logout} disabled={!authenticated}>Logout</button>
      {if authenticated then <button onclick={Auth.api.load_user_info}>User Info</button>}
      <button onclick={Auth.api.create_resource} disabled={!authenticated}>create resource</button>
    </div>


m.route(document.body, "/", {
  "/": Auth
})
