import sys
from pathlib import Path

def main(directory_path):
    directory = Path(directory_path)
    for file in directory.iterdir():
        if file.suffix == '.json':
            print(file)

if __name__ == '__main__':
    main(sys.argv[1])
