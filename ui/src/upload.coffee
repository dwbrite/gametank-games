import Api from './api'

Upload =
  contributors: [] # State for contributors
  game_name: "" # Game name
  description: "" # Game description
  rom: null # ROM file

  view: ->
    <div className="upload-page">
      <h1>Upload Page</h1>
      <div className="form-group">
        <label for="name">Name:</label>
        <input
          type="text"
          id="name"
          name="name"
          placeholder="Enter the name"
          value={Upload.game_name}
          oninput={(e) => Upload.game_name = e.target.value}
        />
      </div>
      <div className="form-group">
        <label for="description">Description:</label>
        <textarea
          id="description"
          name="description"
          placeholder="Enter a description"
          value={Upload.description}
          oninput={(e) => Upload.description = e.target.value}>
        </textarea>
      </div>
      <div className="form-group">
        <label for="rom">ROM File:</label>
        <input
          type="file"
          id="rom"
          name="rom"
          accept=".gtr"
          onchange={(e) => Upload.rom = e.target.files[0]}
        />
      </div>
      <div className="contributors-section">
        <h2>Contributors</h2>
        { Upload.contributors.map (contributor, index) =>
          <div className="contributor" key={index}>
            <input
              type="text"
              placeholder="Contributor Name"
              value={contributor.name}
              oninput={(e) => Upload.updateContributor(index, "name", e.target.value)}
            />
            <select
              value={contributor.role}
              onchange={(e) => Upload.updateContributor(index, "role", e.target.value)}>
              <option value="">Select Role</option>
              <option value="developer">Developer</option>
              <option value="designer">Designer</option>
              <option value="tester">Tester</option>
            </select>
            <button type="button" onclick={() => Upload.removeContributor(index)}>Remove</button>
          </div>
        }
        <button type="button" onclick={Upload.addContributor}>Add Contributor</button>
      </div>
      <button onclick={Upload.submitForm}>Submit</button>
    </div>


  # Add a new contributor to the list
  addContributor: ->
    Upload.contributors.push({ name: "", role: "" })

  # Update a contributor's information
  updateContributor: (index, field, value) ->
    Upload.contributors[index][field] = value

  # Remove a contributor from the list
  removeContributor: (index) ->
    Upload.contributors.splice(index, 1)

  # Handle form submission
  submitForm: (e) ->
    e.preventDefault() # Prevent default button behavior
    if not Upload.game_name or not Upload.description
      alert("All fields are required!")
      return

    fileToByteArray = (file) ->
      new Promise (resolve, reject) ->
        reader = new FileReader()
        reader.onload = (event) ->
          arrayBuffer = event.target.result
          resolve(Array.from(new Uint8Array(arrayBuffer))) # Convert to byte array
        reader.onerror = (error) -> reject(error)
        reader.readAsArrayBuffer(file)

    rom_array = await fileToByteArray(Upload.rom);

    # Create the game via API
    Api.create_game Upload.game_name, Upload.description, rom_array
      .then (response) ->
        alert("Game created successfully!")
        console.log(response)
        # Reset form state
        Upload.resetForm()
      .catch (error) ->
        alert("Failed to create game.")
        console.error(error)

  # Reset form state
  resetForm: ->
    Upload.game_name = ""
    Upload.description = ""
    Upload.rom = null
    Upload.contributors = []


export default Upload