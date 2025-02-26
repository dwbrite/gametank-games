import {update_rom_data} from './bin/gametank-emu-rs'

RustEmu =
  fetchAndLoadROM: (game_id)->
    try
      response = await fetch "http://localhost:41123/api/games/#{game_id}"
      throw new Error "HTTP error! Status: #{response.status}" unless response.ok

      json = await response.json()
      throw new Error "Invalid ROM data received" unless json.game_rom? and Array.isArray(json.game_rom)

      uint8Array = new Uint8Array json.game_rom
      update_rom_data uint8Array
      console.log "Sending ROM: #{uint8Array.length} bytes"
    catch error
      console.error "Failed to load ROM:", error
      
  oncreate: (vnode) ->
    # Move the canvas from the shadow DOM into the emulator's DOM.
    shadowHost = document.getElementById("shadow-host")
    if shadowHost? and shadowHost.shadowRoot?
      canvas = shadowHost.shadowRoot.querySelector("#gt-canvas")
      if canvas?
        # Append the canvas to this component's DOM.
        vnode.dom.appendChild(canvas)
        @canvas = canvas
        console.log "Canvas moved into emulator."
      else
        console.error "Canvas not found in shadow DOM."
    else
      console.error "Shadow host or its shadowRoot not found."

    @gameId = vnode.attrs.gameId
    RustEmu.fetchAndLoadROM(@gameId)
    
  onbeforeremove: ->
    # Move the canvas back into the shadow DOM.
    shadowHost = document.getElementById("shadow-host")
    if shadowHost? and shadowHost.shadowRoot? and @canvas?
      shadowHost.shadowRoot.appendChild(@canvas)
      console.log "Canvas returned to shadow DOM."
    else
      console.error "Failed to move canvas back to shadow DOM."
  
  view: ->
    <div>
    </div>

export default RustEmu
