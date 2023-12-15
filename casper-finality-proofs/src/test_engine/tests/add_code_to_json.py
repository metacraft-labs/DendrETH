import os
import json

folder_path = './prove_finality'  # Replace with the actual path to your folder

# Iterate through each file in the folder
for filename in os.listdir(folder_path):
    file_path = os.path.join(folder_path, filename)

    # Check if the file is a JSON file
    if filename.endswith('.json'):
        # Read the existing JSON content from the file
        with open(file_path, 'r') as file:
            data = json.load(file)

        # Add your code to the JSON data
        data[",previous_justified_checkpoint"] = {
            "epoch": 0,
            "root": "0x0000000000000000000000000000000000000000000000000000000000000000"
        }
        data["current_justified_checkpoint"] = {
            "epoch": 123456788,
            "root": "0x0000000000000000000000000000000000000000000000000000000000000000"
        }
        data["finalized_checkpoint"] = {
            "epoch": 0,
            "root": "0x0000000000000000000000000000000000000000000000000000000000000000"
        }

        # Write the updated JSON content back to the file
        with open(file_path, 'w') as file:
            json.dump(data, file, indent=4)

print("Code added to all JSON files in the folder.")
