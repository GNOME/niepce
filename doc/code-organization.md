Code organization


Here are the different directories for the source code:

* crates - internal Rust crates.
    * npc-fwk - the framework in Rust. Replaces C++ code in src/fwk.
    * npc-engine - the engine in Rust. Replaces C++ code in src/engine.
* src
    * fwk - the framework
        * utils - utilities
        * toolkit - the UI toolkit (Gtkmm based)
            * widgets - reusable widgets
    * engine - the backend engine
        * db - the database storage
        * library - the library ("server" side). Relies on DB
                    for the data storage.
    * ncr - Niepce Camera RAW: the RAW processing engine
    * niepce - The application
        * ui - the UI code
        * modules - The different application modules
            * darkroom - The darkroom module
            * map - The map module
