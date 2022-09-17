# Documentation

## Todo

RPC Monitoring:

- [ ] Machine id
  - [x] Autogenerate 8 byte machine id
  - [x] Store it locally so that users may transfer or change it
  - [ ] Ensure that machine id is unique by checking the db
- [ ] Add token authentication middleware
- [X] Initial State Transfer
  - [x] Transfer static CPU data
  - [x] Refactor process_event to only perform one query per event
  - [X] Assemble and return data in db to machine
- [ ] Refactor to compile-time sqlx queries
  - [ ] Refactor `sqlx::query` to `sqlx::query!`
  - [ ] Automatic start of localsetup if not already running
  - [ ] Refactor individual updates into own functions
- [ ] ? Register a new machine if not exists

General:

- [ ] Test strategy
  - [ ] Introduce traits for testing
  - [ ] Refactor side-effects to beginning of program
  - [ ] Add tests for individual parts
- [ ] Logging framework
  - [ ] Remove debug printing
- [ ] Refactoring
  - [x] Put code into own crates
  - [ ] Reorganize code

## Edge-Cases

- A single machine sends data from two processes
  - [ ] Rate limiting check for machine
- Another machine sends data in the name of an existing machine
  - [ ] Rate limiting check makes it a bit harder to exploit
  - [ ] Frequent ip address change check
- A machine sends data faster than allowed
  - [ ] Rate limiting check for machine

## Unclear

- Use streaming for monitoring
  - Let server ping the client?
    - no bidirectional stream recommended since keeping many connections open
      will have with many connections a severe impact
  - Performance impact many streaming connection?
    - 40 kB memory per channel
    - 80 kB max recommended payload
    - unary streaming is an option for testing
    - no bidir streaming since keeping many different connections open is
      definitely worse than performing a handshake every X minutes
- Try eBPF for measurements if possible?

## Unhappy 😢

- Generally:
  - Use more generics to handle conversions and other interfaces
- `protocol` package
  - Uses `sqlx` to modify types but this should happen in `server`
    where `database.rs` is located
  - Manually implementing `sqlx` trait `FromRow` for protobuf classes
- `tc_core`
  - `u64_to_i64` is a silly hotfix and could be done nicer
  - Could be ripped apart into respective parts
- `client`
  - Code structure has grown too complex over time (could be simpler)
  - Spawning tasks is a bit hidden and should be more explicit
  - Clean up tasks in a nicer way
  - Error handling is not clean
- `server`
  - Revamp function calls to generics if reasonable
  - Rip apart long match
