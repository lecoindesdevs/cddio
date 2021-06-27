from discord.ext import commands
import os

token, prefix = os.environ['TOKEN'], os.environ['PREFIX']

bot = commands.Bot(command_prefix=prefix)

@bot.event
async def on_ready():
    print(f'I\'ve logged in as {bot.user}.')

bot.run(token)