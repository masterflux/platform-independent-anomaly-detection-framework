import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.cm as cm
import numpy as np

def plot_mean_f1(df):
    mean_f1 = df.groupby("algorithm")["f1"].mean().sort_values()
    plt.figure(figsize=(8,4))
    plt.bar(mean_f1.index, mean_f1.values)
    plt.xticks(rotation=45, ha="right")
    plt.xlabel("Algorithm")
    plt.ylabel("Mean F1 score")
    plt.title("Average F1 by Change‐Point Algorithm")
    plt.tight_layout()

def plot_f1_box(df):
    algs = df["algorithm"].unique()
    data = [ df.loc[df["algorithm"]==alg, "f1"] for alg in algs ]
    plt.figure(figsize=(8,5))
    plt.boxplot(data, labels=algs, showmeans=True)
    plt.xticks(rotation=45, ha="right")
    plt.ylabel("F1 score")
    plt.title("Distribution of F1 scores by Algorithm")
    plt.tight_layout()

def plot_f1_vs_cover(df):
    algs = df["algorithm"].unique()
    colors = cm.get_cmap("tab10", len(algs))
    plt.figure(figsize=(6,6))
    for i, alg in enumerate(algs):
        sub = df[df["algorithm"]==alg]
        plt.scatter(sub["cover"], sub["f1"],
                    label=alg, alpha=0.7, s=40, color=colors(i))
    plt.legend(bbox_to_anchor=(1.05,1), loc="upper left")
    plt.xlabel("Covering")
    plt.ylabel("F1 score")
    plt.title("F1 vs Covering by Algorithm")
    plt.tight_layout()

def main():
    df = pd.read_csv("accuracy.csv")
    plot_mean_f1(df)
    plot_f1_box(df)
    plot_f1_vs_cover(df)
    plt.show()

if __name__ == "__main__":
    main()
