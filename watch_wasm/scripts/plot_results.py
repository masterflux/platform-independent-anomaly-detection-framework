import pandas as pd
import matplotlib.pyplot as plt

def main():
    
    df = pd.read_csv("results/size_accuracy.csv")

    plt.figure(figsize=(10, 6))
    
    df.boxplot(column="accuracy_%", by="algorithm", rot=45)
    plt.title("Detection Accuracy Distribution by Algorithm")
    plt.suptitle("")  
    plt.ylabel("Accuracy (%)")
    plt.xlabel("")
    plt.tight_layout()
    plt.savefig("results/accuracy_boxplot.png", dpi=300)
    plt.close()

    
    mean_acc = df.groupby("algorithm")["accuracy_%"].mean().sort_values()
    plt.figure(figsize=(8, 4))
    mean_acc.plot(kind="barh")
    plt.title("Mean Detection Accuracy by Algorithm")
    plt.xlabel("Mean Accuracy (%)")
    plt.ylabel("")
    plt.tight_layout()
    plt.savefig("results/mean_accuracy.png", dpi=300)
    plt.close()

    print("→ Plots saved to results/accuracy_boxplot.png and results/mean_accuracy.png")

if __name__ == "__main__":
    main()
