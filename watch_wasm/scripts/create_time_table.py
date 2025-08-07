import pandas as pd

def create_comprehensive_table():
    """
    Creates a comprehensive CSV table showing execution times for all datasets,
    algorithms, and platforms.
    """
    
    
    windows_path = 'results/win/results-win.csv'
    rasp_path = 'results/rasp/results-rasp.csv'
    ubuntu_path = 'results/Ubuntu/results-ubu.csv'
    
    try:
        
        print("Reading CSV files...")
        win_df = pd.read_csv(windows_path)
        rasp_df = pd.read_csv(rasp_path)
        ubu_df = pd.read_csv(ubuntu_path)        
        
        win_data = win_df[['dataset', 'method', 'time_ms']].copy()
        rasp_data = rasp_df[['dataset', 'method', 'time_ms']].copy()
        ubu_data = ubu_df[['dataset', 'method', 'time_ms']].copy()
        
        win_data.rename(columns={'time_ms': 'Windows (ms)'}, inplace=True)
        rasp_data.rename(columns={'time_ms': 'Raspberry Pi (ms)'}, inplace=True)
        ubu_data.rename(columns={'time_ms': 'Ubuntu (ms)'}, inplace=True)
        
        print("Merging data from all platforms...")
        merged_df = win_data.merge(ubu_data, on=['dataset', 'method'], how='outer')
        merged_df = merged_df.merge(rasp_data, on=['dataset', 'method'], how='outer')
        
        merged_df.sort_values(['dataset', 'method'], inplace=True)
        
        final_df = merged_df.rename(columns={'dataset': 'Dataset', 'method': 'Algorithm'})
        
        output_path = 'comprehensive_time_table.csv'
        final_df.to_csv(output_path, index=False)
        print(f"\nTable saved to: {output_path}")
        
        print("\nFirst 10 rows of the table:")
        print(final_df.head(10))
        
        print(f"\nTotal rows in table: {len(final_df)}")
        
        return final_df
        
    except FileNotFoundError as e:
        print(f"Error: Could not find file - {e}")
        print("Please ensure the CSV files are in the correct locations:")
        print("  - results/win/results-win.csv")
        print("  - results/win/results-rasp.csv")
        print("  - results/win/results-ubu.csv")
        return None
    except Exception as e:
        print(f"Error occurred: {e}")
        return None

if __name__ == "__main__":
    df = create_comprehensive_table()