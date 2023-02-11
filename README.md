# WLMatrix
WLMatrix is a **translation layer** between **MSNP18** & **the Matrix API**. The project started originally in 2020 in C++ but was rewritten entirely in **Rust** leveraging the **Matrix Rust SDK**.
*The goal is to compile WLMatrix as a .dll library and inject it in the process of MsnMsgr.exe. Redirect all the calls to localhost and voilà, you got a new backend and an amazing client relevant again !*
## What's sort of working now
- Redirection to localhost
 - User/Password login
 - Profile pic update
 - Display name update
 - Presence (kinda working, WLM is not really compatible with the Matrix way of handling presence currently)
 - 1v1 DM Rooms
 - Text messages
 - File upload (only upload from WLM to Matrix currently, no receive yet)
 - Multiple Points of Presence (showing all your logged in sessions)
 - E2E encryption (thanks to the SDK heeheehee, WLM more secure than facebook messenger, what a time to be alive) :B

## What's next

 - File upload (Matrix -> WLM)
 - Contact profile pic
 - Custom events for WLM's specific messages (like nudges & winks)
 - Try to map Circles to non DM Rooms
 - Handle DM rooms with more than two people

## Components
There are three distinct components making WLMatrix tick:

 1. **Notification Server** - TCP Server listening on port 1863: This server handles the meat of MSNP18.
 2. **Switchboard Server** (or Mixer) - TCP Server listening on port 1864: This server handles rooms. One room per TCP connection.
 3. **HTTP Server**: This server handles all the SOAP Services used by the client (and various bits and bops)
	 - **The Address Book**: Handles the Contact List
	 - **Sharing Service**: Handles the membership lists (Who has added you back, Who did you add in your contact list)
	 - **Storage Service**: Handles updating and getting your profile
	 - **RST2, Request Security Token**: The SOAP Service responsible for authenticating the client

## Special Thanks
 - The Escargot Project
 - Luis Mariano Guerra and his project Emesene
 - Pang Wu, Ethem Evlice, Scott Werndorfer & The MSNPSharp Team 
 - All the people that saved documentation and packet capture from the time
 - The Matrix Foundation & the team behind the Rust SDK
