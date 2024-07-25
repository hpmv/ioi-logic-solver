import base64
import json

puzzles = json.load(open('Puzzles.json', 'rb'))['puzzles']

grids = []

for puzzle in puzzles:
    if puzzle['puzzleType'] == 'logicGrid':
        data = None
        try:
            if 'pdata' not in puzzle:
                serialized = json.loads(puzzle['serialized'])
                data = base64.b64decode(serialized['BinaryData'])
                difficulty = 0
            else:
                data = base64.b64decode(puzzle['pdata'])
                difficulty = puzzle['difficulty'] if 'difficulty' in puzzle else 0
        except Exception as e:
            print('Error', puzzle['pid'], e)
            raise e

        grids.append((puzzle['pid'], difficulty, data))


class DecodedGrid:
    def __init__(self, pid, difficulty, kind, rows, cols):
        self.pid = pid
        self.difficulty = difficulty
        self.kind = kind
        self.rows = rows
        self.cols = cols
        self.topology = []
        self.rules = []
        self.solution = None
        self.error = None
        self.remainder = None

    def add_topology(self, topology):
        self.topology.append(topology)

    def add_rule(self, rule):
        self.rules.append(rule)
        
    def to_json(self):
        return {
            'pid': self.pid,
            'difficulty': self.difficulty,
            'kind': self.kind,
            'rows': self.rows,
            'cols': self.cols,
            'topology': self.topology,
            'rules': self.rules,
            'solution': self.solution,
            'error': str(self.error),
            'remainder': self.remainder.hex()
        }


class Decoder:
    def __init__(self, pid, difficulty, data):
        self.pid = pid
        self.difficulty = difficulty
        self.data = data
        self.pos = 0

    def read(self):
        self.pos += 1
        return self.data[self.pos - 1]
    

    def read_varint(self):
        result = 0
        while True:
            b = self.read()
            result |= (b & 0x7f)
            if not (b & 0x80):
                break
            result <<= 7
        return result

    def decode(self):
        self.read_varint()
        kind = self.read_varint()
        if kind > 0:
            return None
        self.read_varint()
        rows = self.read_varint()
        cols = self.read_varint()
        grid = DecodedGrid(self.pid, self.difficulty, kind, rows, cols)

        try:
            topology_count = self.read_varint()
            for _ in range(topology_count):
                grid.add_topology(self.read_topology())
            
            rule_count = self.read_varint()
            for _ in range(rule_count):
                grid.add_rule(self.read_rule())

            grid.solution = self.read_solution(rows, cols)
            
            grid.remainder = self.data[self.pos:]
            return grid
        except Exception as e:
            grid.error = e
            grid.remainder = self.data[self.pos:]
            return grid
    
    def read_topology(self):
        kind = self.read_varint()
        if kind == 0:
            count = self.read_varint()
            merges = []
            for _ in range(count):
                merges.append(self.read_varint())
            return ('merge', merges)
        elif kind == 1:
            count = self.read_varint()
            holes = []
            for _ in range(count):
                holes.append(self.read_varint())
            return ('hole', holes)
        else:
            raise Exception("Unsupported topology kind {}".format(kind))
    
    def read_rule(self):
        kind = self.read_varint()
        if kind == 0:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append(self.read_varint())
            return ('light', cells)
        elif kind == 1:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append(self.read_varint())
            return ('dark', cells)
        elif kind == 2:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append((self.read_varint(), self.read_varint()))
            return ('area', cells)
        elif kind == 3:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append((self.read_varint(), self.read_varint()))
            return ('viewpoint', cells)
        elif kind == 4:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append((self.read_varint(), self.read_varint(), self.read_varint()))
            return ('dart', cells)
        elif kind == 5:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append(self.read_varint())
            return ('galaxy', cells)
        elif kind == 6:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append((self.read_varint(), self.read_varint()))
            return ('lotus', cells)
        elif kind == 7:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append((self.read_varint(), self.read_varint()))
            return ('myopia', cells)
        elif kind == 8:
            count = self.read_varint()
            cells = []
            for _ in range(count):
                cells.append((self.read_varint(), self.read_varint()))
            return ('letters', cells)
        elif kind == 0x40:  # ban pattern
            count = self.read_varint()
            patterns = []
            for _ in range(count):
                r = self.read_varint()
                c = self.read_varint()
                num_bytes = (r * c + 3) // 4
                data = [self.read() for _ in range(num_bytes)]
                pattern = []
                pattern_data = []
                for i in range(r):
                    row = ''
                    for j in range(c):
                        index = i * c + j
                        byte_index = index // 4
                        shift = 6 - 2 * (index % 4)
                        cell = (data[byte_index] >> shift) & 3
                        if cell == 0:
                            row += 'L'
                        elif cell == 1:
                            row += 'D'
                        else:
                            row += ' '
                        pattern_data.append(cell)
                    pattern.append(row)
                patterns.append((r, c, pattern, pattern_data))
            return ('ban_patterns', patterns)
        elif kind == 0x41:
            return ('connect_all_light', )
        elif kind == 0x42:
            return ('connect_all_dark', )
        elif kind == 0x43:
            return ('one_symbol_per_light', )
        elif kind == 0x44:
            return ('one_symbol_per_dark', )
        elif kind == 0x45:
            return ('light_shapes_distinct', )
        elif kind == 0x46:
            return ('dark_shapes_distinct', )
        elif kind == 0x47:
            num = self.read_varint()
            return ('light_area', num)
        elif kind == 0x48:
            num = self.read_varint()
            return ('dark_area', num)
        elif kind == 0x49:
            return ('light_shapes_same', )
        elif kind == 0x4a:
            return ('dark_shapes_same', )
        elif kind == 0x7f:
            data = [self.read() for _ in range(10)]  #??
            return ('unknown_rule_0x7f', data)
        else:
            raise Exception("Unsupported rule kind {}".format(kind))
    
    def read_solution(self, rows, cols):
        kind = self.read_varint()
        if kind == 1:
            bytes_needed = (rows * cols + 7) // 8
            data = [self.read() for _ in range(bytes_needed)]
            pattern = []
            pattern_data = []
            for i in range(rows):
                row = ''
                for j in range(cols):
                    index = i * cols + j
                    byte_index = index // 8
                    shift = 7 - (index % 8)
                    cell = (data[byte_index] >> shift) & 1
                    if cell == 0:
                        row += 'L'
                    else:
                        row += 'D'
                    pattern_data.append(cell)
                pattern.append(row)
        elif kind == 2:
            num_bytes = (rows * cols + 3) // 4
            data = [self.read() for _ in range(num_bytes)]
            pattern = []
            pattern_data = []
            for i in range(rows):
                row = ''
                for j in range(cols):
                    index = i * cols + j
                    byte_index = index // 4
                    shift = 6 - 2 * (index % 4)
                    cell = (data[byte_index] >> shift) & 3
                    if cell == 0:
                        row += 'L'
                    elif cell == 1:
                        row += 'D'
                    else:
                        row += ' '
                    pattern_data.append(cell)
                pattern.append(row)
        else:
            raise Exception("Unsupported solution kind {}".format(kind))
        return (kind, pattern, pattern_data)

decoded_db = []

for (pid, difficulty, data) in grids:
    decoder = Decoder(pid, difficulty, data)
    decoded = decoder.decode()
    if decoded is not None:
        print('-----------------------')
        if decoded.error:
            print('Error', pid, decoded.error)
        print(pid, decoded.kind, decoded.rows, decoded.cols, decoded.topology, decoded.rules, decoded.solution, decoded.remainder.hex())
        decoded_db.append(decoded.to_json())

json.dump(decoded_db, open('decoded.json', 'w'))