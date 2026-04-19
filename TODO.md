# Stuff to do

## Installer

- [X] Installs Tachyon and all the necessary files to make it run.

## Launching the server

- [X] Make patching solution compatible with third party MSN servers (escargot, crosstalk, etc)
- [X] Create launcher to start msnmsgr.exe with zathras imported. (Remove all unknown dll imports)
- [X] Config file for both zathras and tachyon to share the port, the logging, etc...

## Landing Page

- [ ] Do a download setup landing page

## MVP Features
- [X] Simplify the contact list. Show rooms, not contacts.
- [X] Send Text Messages
- [X] Add, Remove contacts
- [X] E2E Device verification
- [X] MSN Today Page
- [ ] Memory management of heap allocated objects
  - [X] DropGuard for Clients
  - [ ] Outer select! on spawned tasks to handle kill signals
  - [ ] Period sweep of GlobalData for unused objects
- [ ] MSNP Errors
- [ ] Anonymize logs

## MMP Features
- [ ] Testing
- [ ] OIMs
- [ ] P2PV2
  - [ ] File Transfer
  - [ ] Avatar Transfer
  - [ ] Voice Messages
- [ ] Search for contacts
- [ ] Polish Web Pages
- [ ] Recovery Key Setup Page

## Next
- [ ] Refactor & Consolidate conversions in Tachyon Client. (Send Message, Accept Message, etc...)
- [ ] zathras.dll opens the browser for tachyon login, not the server. (React to msidcrl login function)
- [ ] OAUTH Support
- [ ] Groupchats (msn spaces or ephemeral rooms)
- [ ] Spaces to category mapping
- [ ] Status & Presence using new Account Data
- [ ] Image sharing
- [ ] Voice calls
