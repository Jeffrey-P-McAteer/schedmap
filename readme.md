
# SchedMap

The swiss army knife of all scheduling.

# Development

SchedMap is an HTTP webserver and CLI tool written in Rust.
You will need the `cargo` utility to compile and run the code. Read more at https://rustup.rs/.

`cargo run` will run the program; without arguments this prints the help text of SchedMap.
Arguments may be added like `cargo run server --app-port=4321`

`cargo build --release` will build an optimized binary at `target/release/schedmap`.

SchedMap also has a client component; the `src/www/` directory holds the largest pieces of the web app used to display scheduling state information to a browser.
The browser capabilities can also be done via CLI by calling `schedmap client [other arguments]`, but this is still incomplete.

# Directory layout

Server code is in `src/`, browser client app code is mostly in `src/www`, but just because a file is under `www/` does not mean it will be served, a route must be added in `src/routes.rs` which serves the resource.

`test-assets` should be used to store example data used during testing new capabilities.

# Plans

At the moment the codebase is a wreck; Jeffrey has chosen to attack the most difficult problems first (websocket server, broadcast capabilities, shared mutable data), and because of this the application does nothing useful except show proof of concept message passing.

Immediate TODOs include:

 - [ ] Uploading an arbitrary SVG map from both CLI and Browser
 - [ ] Throw some "user X has touched room Y" logic in there, test with USB smartcard reader.
 - [ ] Flush out setup stuff - we should have lists of roles and employees, as well as global constraints (eg warn if employee is scheduled for >40hrs in any given week).


