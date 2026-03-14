# Stuff to do

- [ ] Keep it simple, stupid.

## Installer

- [ ] Install WLM & WLM Contacts
  - Check if there is a silent mode on the setup.exe
- [ ] Patch both msnmsgr.exe & wlcomm.exe with the detour library
- [ ] Copy the db dll broken by Windows 11 24H2 (by swapping it with an older version from a previous windows)

## Launching the server

- [ ] Figure out how do we start the server
  - Start in the detour DLL ?
  - Watchdog that spawns both msnmgr.exe and tachyon.exe and monitors both processes
  - Inject dll in the watchdog instead of binary patching the import ?
  - We still need to patch wlcomm.exe if we do that
- [ ] Figure out how do we configure the server ports
  - Detour dll needs to know the ports, let's use static ports that aren't used by anything else.

## MVP Features
- [X] Simplify the contact list. Show rooms, not contacts.
- [X] Send Text Messages
- [ ] E2E Device verification
- [ ] Add, Remove contacts
- [ ] Search for contacts
- [ ] Avatar Transfer

## Nice to have
- [ ] Groupchats (msn spaces or ephemeral rooms)
- [ ] Spaces to category mapping
- [ ] File sharing working
- [ ] Status & Presence using new Account Data
- [ ] Image sharing
- [ ] Voice calls