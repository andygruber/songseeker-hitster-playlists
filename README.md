# songseeker-hitster-playlists

CSV files containing YouTube links for Hitster playing cards. This allows you to access the music for your game nights directly via YouTube.

## Credits & Background
This project is based on the original work by **Andreas Gruber**.

### Why this version?
The original data had become outdated and contained many dead links. This repository provides an updated version of the data to ensure your game nights run smoothly. 

I replaced the original Python code with **Rust**. Since I'm not a Python programmer and the original script was quite slow (as it didn't utilize the official API), this version is built for speed and reliability.

## Prerequisites
To run the scripts or update the data yourself, you need your own **YouTube API Key**.

1. Create a project in the [Google Cloud Console](https://console.cloud.google.com/).
2. Enable the **YouTube Data API v3**.
3. Create a file named `.env` in the root directory.
4. Add your key to the file:
   ```env
   API_KEY=YOUR_YOUTUBE_API_KEY_HERE
   ```

## Content
The data includes all German Hitster cards from the original game, the [official Hitster website](https://hitstergame.com/de-de/), and the public playlist on [Spotify: Hitster - Deutsch](https://open.spotify.com/playlist/26zIHVncgI9HmHlgYWwnDi).

* **`playlists.csv`**: A list of playlist files and the names of the corresponding game editions.
* **CSV Files**: These contain the direct mapping of Hitster cards to functioning YouTube links.
