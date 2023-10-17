import json
import sys

def sum_values_from_json(json_file):
    with open(json_file) as f:
        data = json.load(f)
        inputs = data["inputs"]
        outputs = data["outputs"]

        a = int(inputs["a"], 16)
        b = int(inputs["b"], 16)
        c = int(outputs["c"], 16)

        result = a + b

        if result != c:
            raise ValueError(f"The sum of {a} and {b} is {result}, but {c} was expected.")

        return result

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python ref.py <path_to_json_file>")
        sys.exit(1)

    result = sum_values_from_json(sys.argv[1])
    print("Sum is correct:", result)
