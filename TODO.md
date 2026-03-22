# Stuff to do

## Installer

- [ ] Install WLM in silent mode `wl-setup.exe /q /NOToolbarCEIP /NOhomepage /nolaunch /nosearch /AppSelect:Messenger`
- [ ] create wlcomm-tachyon.exe and add new COM class in the registry
- [ ] add zathras.dll to wlcomm-tachyon import table
- [ ] create Tachyon idcrl environment in the registry
- [ ] Copy the db dll broken by Windows 11 24H2 (by swapping it with an older version from a previous windows)

## Launching the server

- [X] Make patching solution compatible with third party MSN servers (escargot, crosstalk, etc)
- [ ] Create launcher to start msnmsgr.exe with zathras imported. (Remove all unknown dll imports)
- [ ] Config file for both zathras and tachyon to share the port, the logging, etc...

## Landing Page

- [ ] Do a download setup landing page

## MVP Features
- [X] Simplify the contact list. Show rooms, not contacts.
- [X] Send Text Messages
- [X] Add, Remove contacts
- [ ] E2E Device verification
- [X] MSN Today Page
- [ ] MSNP Errors
- [ ] Testing
- [ ] Anonymize logs

# First Release
- [ ] OIMs
- [ ] Search for contacts
- [ ] File Transfer
- [ ] Avatar Transfer
- [ ] Voice Messages

## Next
- [ ] Groupchats (msn spaces or ephemeral rooms)
- [ ] Spaces to category mapping
- [ ] Status & Presence using new Account Data
- [ ] Image sharing
- [ ] Voice calls
