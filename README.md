# RSBWAPI
## Getting Started
If you're looking for documentation, sorry - we ain't there yet.
The original documenation for [bwapi](https://bwapi.github.io/) should be good enough to get started.
Other than that, you can take a look at [ExampleBot](https://github.com/Bytekeeper/rsbwapi/tree/master/example_bot).

Disclaimer: The API is not yet stabilized at all. Feel free to suggest changes or provide PRs. Generally I will try to keep it close to BWAPI, but will deviate if it makes sense. Ie. errors are not stored in a global error variable, rather they are return as `BWResult`.

## Goal
The project intends to provide a client side implementation for a BWAPI server. It cannot be used to create "module" type bots. 