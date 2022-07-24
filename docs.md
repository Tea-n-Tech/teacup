# Documentation

## Todo

RPC Monitoring:

- [ ] Machine id
  - [x] Autogenerate 8 byte machine id
  - [x] Store it locally so that users may transfer or change it
  - [ ] Ensure that machine id is unique by checking the db
- [ ] Add token authentication middleware
- [ ] Initial State Transfer
  - [x] Transfer static CPU data
  - [x] Refactor process_event to only perform one query per event
  - [ ] Assemble and return data in db to machine
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
- [ ] Refactoring
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
  - Performance impact many streaming connection?
