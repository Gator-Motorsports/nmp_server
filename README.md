# New Member Project Data Handler Server
This is the server that you will be exchanging TCP packets with for the new member project.

# Rust Installation
Go to [Rust's website](https://www.rust-lang.org/tools/install) and follow their installation instructions.

# Installation
You can install the server just by cloning it.

```
git clone https://github.com/Gator-Motorsports/nmp_server.git
cd nmp_server
```

# Running the server
You can run the server from the terminal by entering: `cargo run --release --bin nmp_server -- -t ip_addr`.

Example ip address format: 127.0.0.1:5000

Note that while the server will be active, you will have to run a seperate program attached in order to populate it with test data.

# Populating the server with data
Even though the server will be running. If you want test data, you can run.

`cargo run --release --bin nmp_tests -- -p program_name -t ip_addr`

There's a couple of different test programs that will populate the server differently. 

`100hz`: produces a single static signal with name "100hz" at just under 100 samples per second

`1khz`: produces a single static signal with name "1khz" at just under 1000 samples per second

`1khz_o`: produces a sine wave signal with name "1khz" at just under 1000 samples per second

`1khz_4`: produces 4 single static signal with name "1khz" at just under 1000 samples per second

More test programs are currently a work in progress, feel free to make your own on your local clone of the server.
