# songseeker-hitster-playlists
CSV files containing Youtube links to the Hitster playing cards

Information of all German Hitster cards from the original game, the [Hitster website](https://hitstergame.com/de-de/) and from the public playlist on [Spotify: Hitster - Deutsch](https://open.spotify.com/playlist/26zIHVncgI9HmHlgYWwnDi).

The file `playlists.csv` contains a list of playlist files and the names of the corresponding games.

## Checking YouTube links

Running `verifyYoutubeLinks.py` requires Python and the dependencies listed in `requirements.txt`. Create and activate a virtual environment, install the dependencies, and run the script with a CSV file:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -r requirements.txt
python verifyYoutubeLinks.py hitster-de.csv --check
```

### CLI options

- `input_file` (required): Path to the CSV file to verify.
- `--output_file <path>`: Writes the updated CSV to this file. The script adds or updates `Youtube-Title` and `Hashed Info`. This must be a different file from `input_file`; using the same file may overwrite the input data.
- `--check`: Checks for mismatches without modifying or writing CSV data. The process exits with status `1` when mismatches are found.
- `--browser_automation`: Uses headless Chrome for additional playback checks. This is slower and detects issues such as unlisted or Premium-only videos.
- `--start_row <number>`: Starts processing at this 1-based row number. The default is `1`.
- `--end_row <number>`: Processes rows through this inclusive 1-based row number. By default, all remaining rows are processed.

For example, update a CSV while checking only rows 10 through 50:

```bash
python verifyYoutubeLinks.py hitster-de.csv --output_file hitster-de-updated.csv --start_row 10 --end_row 50
```
