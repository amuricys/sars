import plotly.express as px
import pandas as pd

df = pd.read_csv('output_file01.csv')

df["initial_num"] = df["initial_num"].astype(str)

fig = px.scatter(df, x = 'log convex perimeter', y ='K' , title='Perimeter Law K', color = 'initial_num')
fig.show()

fig = px.scatter(df, x = 'log outer perimeter', y ='K' , title='Perimeter Law K', color = 'initial_num')
fig.show()

fig = px.scatter(df, x = 'log thickness', y ='K' , title='Perimeter Law K', color = 'initial_num')
fig.show()



