import csv
import time
import hashlib
import requests
import argparse
import sys

# Function to query YouTube for video title using oEmbed
def query_youtube_oembed(video_url):
    oembed_url = f"https://www.youtube.com/oembed?url={video_url}"
    try:
        response = requests.get(oembed_url)
        response.raise_for_status()  # Raises an error for bad responses
        data = response.json()
        title = data['title']
        # You can add more data to hash here if needed
        important_info = title + data['author_name']
        return title, hashlib.sha256(important_info.encode()).hexdigest(), None
    except requests.RequestException as e:
        return None, None, f"Request error or video not found: {e}"

def check_and_update_row(row, current_title, current_hashed_info, check_mode):
    updated = False
    mismatch_found = False
    stored_title = row.get('Youtube-Title', '')
    stored_hashed_info = row.get('Hashed Info', '')

    if stored_title != current_title or stored_hashed_info != current_hashed_info:
        mismatch_found = True
        if not check_mode:
            row['Youtube-Title'] = current_title
            row['Hashed Info'] = current_hashed_info
            updated = True

    return updated, mismatch_found

def process_csv(input_file, output_file, start_row, end_row, check_only=False):
    mismatches = 0
    matches = 0
    current_row_number = 0  # Track the current row number

    with open(input_file, mode='r', newline='', encoding='utf-8') as infile:
        reader = csv.DictReader(infile)
        fieldnames = reader.fieldnames

        if not check_only and output_file:
            outfile = open(output_file, mode='w', newline='', encoding='utf-8')
            if fieldnames.count("Youtube-Title") == 0:
                fieldnames.append("Youtube-Title")
            if fieldnames.count("Hashed Info") == 0:
                fieldnames.append("Hashed Info")
            writer = csv.DictWriter(outfile, fieldnames=fieldnames)
            writer.writeheader()
        else:
            writer = None

        for row in reader:
            current_row_number += 1
            if current_row_number < start_row or (end_row and current_row_number > end_row):
                continue  # Skip rows outside the specified range

            video_url = row.get("URL", "")
            current_title, current_hashed_info, error = query_youtube_oembed(video_url)
            
            if error:
                print(f"Error processing video {video_url}: {error}", file=sys.stderr)
                row["Youtube-Title"] = "ERROR"
            else:
                updated, mismatch_found = check_and_update_row(row, current_title, current_hashed_info, check_only)
                if mismatch_found:
                    cardnr=row.get('Card#', "")
                    print(f"Mismatch found for card# {cardnr}, video url: {video_url}")
                    mismatches += 1
                else:
                    matches += 1
            
            if output_file and not check_only:
                writer.writerow(row)

            time.sleep(1)  # Throttle requests

        print(f"Done. {matches} matches, {mismatches} mismatches.")

        if output_file:
            outfile.close()

    return mismatches

# Main logic for processing arguments and invoking the processing function
if __name__ == "__main__":
    # Setup argparse for command-line arguments
    parser = argparse.ArgumentParser(description="Verify YouTube Links in a CSV file.")
    parser.add_argument('input_file', type=str, help='Input CSV file path')
    parser.add_argument('--output_file', type=str, default=None, help='Output CSV file path (optional in check mode)')
    parser.add_argument('--check', action='store_true', help='Run in check-only mode')
    parser.add_argument('--start_row', type=int, default=1, help='Start row number to process (1-based indexing)')
    parser.add_argument('--end_row', type=int, default=None, help='End row number to process (inclusive, optional)')

    args = parser.parse_args()
    mismatches = process_csv(args.input_file, args.output_file, args.start_row, args.end_row, args.check)
    if args.check and mismatches > 0:
        sys.exit(1)