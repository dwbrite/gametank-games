import {update_rom_data} from './bin/gametank-emu-rs'

RustEmu =
  game_data: null
  scale: 3


  fetchGameData: (vnode, game_id) ->
    try
      response = await fetch "http://localhost:41123/api/games/#{game_id}"
      throw new Error "HTTP error! Status: #{response.status}" unless response.ok

      json = await response.json()
      unless json.game_rom? and Array.isArray(json.game_rom)
        throw new Error "Invalid ROM data received"

      vnode.state.game_data = json
      console.log json
    catch error
      console.error "Failed to fetch game data:", error

  # Load the fetched ROM data into the emulator.
  loadROM: (vnode) ->
    update_rom_data new Uint8Array(vnode.state.game_data.game_rom)

  oninit: (vnode) ->
    await RustEmu.fetchGameData(vnode, vnode.attrs.gameId)
    await RustEmu.loadROM(vnode)
    m.redraw()

  oncreate: (vnode) ->
    # Move the canvas from the shadow DOM into the emulator's DOM.
    shadowHost = document.getElementById("shadow-host")
    if shadowHost? and shadowHost.shadowRoot?
      canvas = shadowHost.shadowRoot.querySelector("#gt-canvas")
      if canvas?
        emulatorContainer = vnode.dom.querySelector ".emulator-container"
        if emulatorContainer?
          emulatorContainer.appendChild canvas
          @canvas = canvas
          console.log "Canvas moved into emulator container."
          canvas.focus()
        else
          console.error "Emulator container not found in vnode DOM."
      else
        console.error "Canvas not found in shadow DOM."
    else
      console.error "Shadow host or its shadowRoot not found."

  onbeforeremove: ->
    # Move the canvas back into the shadow DOM.
    # TODO: hard-reset emulator and pause?
    shadowHost = document.getElementById("shadow-host")
    if shadowHost? and shadowHost.shadowRoot? and @canvas?
      shadowHost.shadowRoot.appendChild(@canvas)
      console.log "Canvas returned to shadow DOM."
    else
      console.error "Failed to move canvas back to shadow DOM."

  scale_up: ->
    currentScale = parseFloat(getComputedStyle(document.body).getPropertyValue("--emu-scale")) or 3
    newScale = Math.min(6, currentScale + 1)
    document.body.style.setProperty "--emu-scale", newScale
    localStorage.setItem "emu-scale", newScale

  scale_down: ->
    currentScale = parseFloat(getComputedStyle(document.body).getPropertyValue("--emu-scale")) or 3
    newScale = Math.max(1, currentScale - 1)
    document.body.style.setProperty "--emu-scale", newScale
    localStorage.setItem "emu-scale", newScale


  view: (vnode)->
    # TODO: if you have permissions, show the edit button?
    <div>
      <div class="rust-emu">
        <div className="emulator-container">
          <div className="emu-game-titlebar">
            <h2>{vnode.state.game_data?.game_name or "..."}</h2>
            <div className="controls">
              <button><span>Edit</span></button>
              <button><span>Download</span></button>
              <button onclick={RustEmu.scale_down}><code>-</code></button>
              <button onclick={RustEmu.scale_up}><code>+</code></button>
            </div>
          </div>
          <div>
            <p>{vnode.state.game_data?.author or "..."}, { (vnode.state.game_data?.created_at.split('T')[0]) or "..." }</p>
          </div>
        </div>
        <div>{vnode.state.game_data?.description or "..."}</div>
        {vnode.attrs.canEdit and
        <button onclick={-> console.log("Edit clicked")}>Edit</button>}
      </div>
    </div>

export default RustEmu
