<p align=center>
<img src="https://github.com/aeoncl/wlmatrix-rust/assets/48886723/7691e064-30bf-4253-99db-62f77b57d9ac"  width="40%" />
</p>

# Tachyon
Tachyon is a **translation layer** between **MSNP18** & **the Matrix API**. 

## What's sort of working now
 - Redirection to localhost
 - User/Password login
 - Profile pics
 - Display names
 - Presence (kinda working, WLM is not really compatible with the Matrix way of handling presence currently)
 - 1v1 DM Rooms
 - Text messages
 - File upload
 - Multiple Points of Presence (showing all your logged in sessions)
 - E2E encryption (kinda, no client validation)
 - Vocal messages
 - Group dms (they appear when someone else talks in them)

## What's next
 - Photo sharing
 - Contact management (Add, remove, etc)
 - Handle all the default matrix messages types (m.notice, etc)
 - Presence updates when presence disabled on homeserver
 - Add & Delete contacts
 - Font Styling for text messages
 - Custom events for WLM's specific messages (like nudges & winks)
 - Try to map Circles to non DM Rooms
 - Handle DM rooms with more than two people
 - Contacts recent activity
 - MSNToday
 - MSNObject stores (Emoticon packs, winks, etc)
 - Custom Games & Activities

## Components
There are three distinct components making Tachyon tick:

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
 - The team behind Windows Live Messenger, especially 14.0: thanks for all my childhood memories and friendships that still carried on to this day
