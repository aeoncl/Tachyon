<p align=center>
<img src="https://github.com/aeoncl/wlmatrix-rust/assets/48886723/7691e064-30bf-4253-99db-62f77b57d9ac"  width="40%" />
</p>

# Tachyon
Tachyon is a work in progress **translation layer** between **MSNP18** & **the Matrix API**. 

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
 - All the people that saved documentation and packet captures at the time
 - The Matrix & Element folks
 - The team behind Windows Live Messenger, especially 14.0: thank you for all the childhood memories
