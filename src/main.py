from discord.ext import commands
import os, cogs_management

token, prefix = os.environ['TOKEN'], os.environ['PREFIX']

bot = commands.Bot(prefix, help_command=None)

cogs_management.add_cogs(bot, './cogs')

bot.run(token)
