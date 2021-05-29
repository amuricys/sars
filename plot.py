import pandas as pd
import plotly.express as px

df = pd.read_csv('output.csv')

fig = px.line(df, x = 'timestep', y = 'energy', title='energy per timestep')
fig.show()