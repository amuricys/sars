import os
import pandas as pd
import plotly.express as px





for out_file in os.listdir("output_files"):

    df = pd.read_csv('output_files/{out_file}')
    fig = px.scatter(df, x = df.iloc[2], y = df.iloc[3], title='Perimeter Law')
    fig.show()
