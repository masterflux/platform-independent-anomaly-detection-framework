#!/usr/bin/env python3
import sys, os, json
# make sure we import their copy:
sys.path.insert(0, os.path.dirname(__file__))
import bocpdms

if __name__ == "__main__":
    # usage: detect_bocpdms.py <data_csv> <params_csv> <dataset_name>
    data_path, params_path, name = sys.argv[1:]
    # load your CSV as a 2D numpy array:
    import numpy as np
    mat = np.loadtxt(data_path, delimiter=",")
    if mat.ndim == 1:
        mat = np.expand_dims(mat, 1)
    # read the params for this dataset:
    prior_a = prior_b = intensity = None
    with open(params_path) as f:
        for L in f:
            parts = L.strip().split(",")
            if parts[0] == name:
                prior_a, prior_b, intensity = map(float, parts[1:4])
                break
    if prior_a is None:
        sys.exit(1)
    # call their detect():
    cps = bocpdms.detect(mat, intensity, prior_a, prior_b)
    # print JSON to stdout:
    print(json.dumps(cps))
