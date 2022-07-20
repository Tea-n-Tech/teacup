# Documentation

## Todo

RPC Monitoring:

- [ ] Register a new machine if not exists
- [ ] Use an 8 byte machine id
- Machine id
  - [x] Autogenerate 8 byte machine id
  - [x] Store it locally so that users may transfer or change it
  - [ ] Ensure that machine id is unique by checking the db
- [ ] Add token authentication middleware

General:

- [ ] Test strategy
- [ ] Logging framework

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
