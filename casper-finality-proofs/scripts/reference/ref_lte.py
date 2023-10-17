import json

def lte_from_file(file_path):
    with open(file_path, 'r') as f:
        data = json.load(f)
        inputs = data['inputs']
        a = int(inputs['a'], 16)
        b = int(inputs['b'], 16)

        if a > b:
            raise ValueError(f"{hex(a)} is not less than or equal to {hex(b)}")

if __name__ == "__main__":
    file_path = 'path/to/your/file.json'  # Replace with the actual file path
    try:
        lte_from_file(file_path)
    except ValueError as e:
        print(f"Error: {e}")
