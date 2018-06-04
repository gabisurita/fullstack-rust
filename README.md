Rust Full Stack Example
=======================

**Highly WIP**

This repository is a proof of concept and a proposal for full stack applications using
[Rust](). It's organized in three basic crates (schema, client and server).

This example is mainly REST oriented, but opinions on using other API designs are very
welcome (I'm specially interested to try GraphQL using [Juniper]()).

Basic Architecture
==================

Schema
------

The schema library contains the shared contract between server and client applications.
You may use it to declare (de)serializable models and types using [Serde](), as well
as traits for each resource.


Client (Frontend)
-----------------

For the client appication, we are using a Webassembly application with javascript
iteroperability. This example depends on the latest versions of [Cargo web]() build tool,
and the component based [Yew framework]().

Notice that we are using native Rust to webassembly compilation
(target `wasm-unknown-unknown`), but one can also use [Emscripten]()
to compile this example to asm.js, and thus not requiring the latest versions
of Firefox or Chrome.

The frontend application is basically composed of an API Service implementing the schema
negotiation and a Component for managing the local state and events.



Server (Backend)
----------------

Finally, the server application is resposible for serving the static frontend application
and serving the API for the given example. It's implemented using [Rocket]() and a simple
[Redis]() list for simplicity.

On the server, you may find view-controllers and repositories for the schema resources.
Notice that on this example we are not splitting them as we only implement a
really simple CRUD API.


```
cargo run
```


Running
=======

First, make sure you have the latest Rust Nightly version, Cargo and Cargo-web.

Run a redis instance for storing the data. If you have docker installed, do:

```
docker run --name redis p 6379:6379 -d redis
```

Next, you may build the frontend application using cargo-web and putting the
resulting files on the static server dir.

```
cd client
cargo-web deploy
cp target/deploy/* ../server/static/
```

Finally, run the server application

```
cd server
cargo run
```

You may try it on `locahlost:8000`. Enjoy.
