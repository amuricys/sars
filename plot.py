import pandas as pd
import plotly.express as px



df = pd.read_csv('output.csv')

fig = px.line(df, x = 'timestep', y = 'energy', title='energy per timestep')
fig.show()

fig = px.line(df, x = 'timestep', y = 'gray matter area', title='gray matter area per timestep')
fig.show()

fig = px.line(df, x = 'timestep', y = 'outer perimeter', title='outer perimeter per timestep')
fig.show()

fig = px.line(df, x = 'timestep', y = 'convex perimeter', title='convex perimeter per timestep')
fig.show()

