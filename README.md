# audio_effects
Audio Effects Processor In Rust

## About

Part of Richard's Adventures in Rust Embedded.

This repo contains a proof of concept library implementation of an audio effects processor roughly modeled on a physical rack of audio processing equipment.

## Goals

Build a library that will be useful in an embedded application and learn the rust programming language.

## Components

The software represents the following components (hierarchically from highest to lowest):

* Rack
* Unit
* Processor
* Input Block, Output Block
* Connection, Buffer
* Connector

## Graph

The library expects connections to form an acyclic graph. No cycle detection is currently provided. Starting nodes in the graph are determined by the absence of input connections. The traversal algorithm was written with the intent of experimenting with multi-threading at some point.

## Example

See ./audio_effects/examples/sinefun.rs for a reasonably complete example.

Try: `./audio_effects$ cargo run --example sinefun`

## Code Layout

The code is layed out into four sub-crates:

* effects - Contains the implementations of the effects processors.
* examples - Self explanitory.
* rack - Contains the graph traversal code.
* shared - Contains code shared by rack, examples and effects.

## TODO 

* More unit tests. 
* Learn more about rust's documentation in code comments and format/update as necessary.
* Write more processors.
* The current implementation of graph traversal is non-trival. Research and see if there are better solutions.
* Muti-threaded processing.
* Cycle detection.
* Prelude Competeness.
* Determine if Box is available in no-std environment. If so replace indexes with Box.
