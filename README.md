# YAFTCLogViewer
Yet Another FTC Log Viewer, A Cross-Platform First Tech Challenge Log Viewer, written in Rust with eGUI.

# Known Issues:
- On Windows, selecting the log file from the Control Hub, Expansion Hub or Driver Hub volume will cause it to not open, 
to fix this, simply copy the file off of the external media onto your disk somewhere, like the documents folder, then select it in the program. 

- On Linux, if you spend too much time in the file dialouge Linux will believe the application has stopped responding and once you select a file it'll warn you about it
just click "Wait" and it'll keep working, this can be fixed using the Async File Browser from the RFD Crate, and will likely be fixed in a future update. 

# Tested Operating Systems
### Windows 11 (25H2):
<img width="1363" height="632" alt="image" src="https://github.com/user-attachments/assets/8e3ef209-2751-44c0-9c13-2a12bf4d5ade" />

### Linux Fedora 44:
<img width="2022" height="1064" alt="image" src="https://github.com/user-attachments/assets/2d08796d-b07a-40ba-a8af-52caf02117b8" />
