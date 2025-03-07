import {update_rom_data} from './bin/gametank-emu-rs'
import Api from "./api"

RustEmu =
  game_data: {}  # Initialize an empty array to hold game data
  loading_rom: true   # Loading state
  loading_metadata: true   # Loading state
  error: null     # Error state

  oninit: (vnode) ->
    RustEmu.loading_metadata = true
    RustEmu.loading_rom = true

    Api.get_game(vnode.attrs.gameId)
      .then (data) ->
        RustEmu.game_data = data
        RustEmu.loading_metadata = false
      .catch (err) ->
        RustEmu.error = err
        RustEmu.loading_metadata = false

    Api.get_game_rom(vnode.attrs.gameId)
      .then (arrayBuffer) ->
        console.log "Received ROM buffer size:", arrayBuffer.byteLength  # Debugging
        rom_data = new Uint8Array(arrayBuffer)  # ✅ Convert ArrayBuffer to Uint8Array
        update_rom_data(rom_data)
        RustEmu.loading_rom = false
      .catch (err) ->
        console.error "ROM Fetch Error:", err
        RustEmu.error = err
        RustEmu.loading_rom = false

#    m.redraw()

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
        <div className="emulator-container"> {
          if RustEmu.loading_metadata
            <div className="loading">Loading games...</div>
          else if RustEmu.error?
            <div className="error">{RustEmu.error}</div>
          else
            <div>
              <div className="emu-game-titlebar">
                <h2>{RustEmu.game_data.game_name}</h2>
                <div className="controls">
                  <button><span>Edit</span></button>
                  <button onclick={
                    -> window.location.href = "/api/games/#{RustEmu.game_data.game_id}/rom";
                  }><span>Download</span></button>
                <button onclick={RustEmu.scale_down}><code>-</code></button>
                <button onclick={RustEmu.scale_up}><code>+</code></button>
              </div>
            </div>
            <div>
              <p>{RustEmu.game_data.author}, { (RustEmu.game_data.created_at.split('T')[0])}</p>
            </div>
            </div>
        } </div>
        <div>{RustEmu.game_data.description}</div>
      </div>
    </div>

export default RustEmu
