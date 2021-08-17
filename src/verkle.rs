/*
https://notes.ethereum.org/@vbuterin/verkle_tree_eip

# Bandersnatch curve order
BANDERSNATCH_MODULUS = \
13108968793781547619861935127046491459309155893440570251786403306729687672801
# Bandersnatch Pedersen basis of length 256
PEDERSEN_BASIS = [....]
VERKLE_NODE_WIDTH = len(PEDERSEN_BASIS)

def group_to_field(point: G1Point) -> int:
    # Not collision resistant -- not usable for Fiat-Shamir
    assert isinstance(point, G1Point)
    if point == bls.Z1:
        return 0
    else:
        return int.from_bytes(serialize(point), 'little') % BANDERSNATCH_MODULUS
    
def compute_commitment_root(children: Sequence[int]) -> int:
    o = bls.Z1
    for generator, child in zip(PEDERSEN_BASIS, children):
        o = bls.add(o, bls.mul(generator, child))
    return group_to_field(o)

def pedersen_leaf(stem: bytes31, value: bytes32) -> int:
    value_lower = value[:16]
    value_upper = value[16:]
    return compute_commitment_root([1, # Leaf marker
                                    int.from_bytes(stem, "little"),
                                    int.from_bytes(value_lower, "little"),
                                    int.from_bytes(value_upper, "little")] +
                                    [0] * 252)

def compute_main_tree_root(data: Dict[bytes32, int],
                           prefix: bytes) -> int:
    # Empty tree: 0
    if len(data) == 0:
        return 0
    # Single element: byte-pack [key, value]
    elif len(data) == 1:
        key, value = list(data.items())[0]
        return value
    else:
        sub_commitments = [
            compute_main_tree_root({
                    key: value for key, value in data.items() if
                    key[:len(prefix) + 1] == prefix + bytes([i])
                }, prefix + bytes([i]))
            for i in range(VERKLE_NODE_WIDTH)
        ]
        return compute_commitment_root(sub_commitments)
        
def compute_verkle_root(data: Dict[bytes32, bytes32]) -> int:
    # Special-case: collapse the bottom layer into commitments
    stems = set(key[:-1] for key in data.keys())
    data_as_stems = {}
    for stem in stems:
        commitment_data = [0] * 256
        for i in range(VERKLE_NODE_WIDTH):
            if stem + bytes([i]) in data:
                commitment_data[i] = pedersen_leaf(stem, data[stem + bytes([i])])
        data_as_stems[stem] = compute_commitment_root(commitment_data)
    return compute_main_tree_root(data_as_stems, b'')
*/
