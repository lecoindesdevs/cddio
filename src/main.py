from discord.ext import commands
from cogs.commands.help_command import HelpCommand
import os, cogs_management, logging

# Sets up logging
logging.basicConfig(level=logging.INFO)

# Gets environment data.
token, prefix = os.environ['TOKEN'], os.environ['PREFIX']

# Creates the bot without default help command.
bot = commands.Bot(command_prefix=prefix, help_command=HelpCommand())

# Adds cogs to the bot.
cogs_management.add_cogs(bot, './cogs')

# Runs bot.
bot.run(token)
