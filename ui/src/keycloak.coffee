import Keycloak from 'keycloak-js';

keycloak = new Keycloak({
  url: 'https://keycloak.dwbrite.com',
  realm: 'gametank-games',
  clientId: 'login-frontend',
})

export default keycloak;
