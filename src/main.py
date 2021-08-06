from discord.ext import commands
from cogs.commands.help_command import HelpCommand
import discord, os, cogs_management, logging, config

# Sets up logging.
logging.basicConfig(level=logging.DEBUG)

# Gets environment data.
config.TOKEN, config.PREFIX = os.environ['TOKEN'], os.environ['PREFIX']

# Initializes default intents.
intents = discord.Intents.default()
intents.members = True

# Creates bot with a custom help command and default intents.
bot = commands.Bot(command_prefix=config.PREFIX, help_command=HelpCommand(), intents=intents)

# Adds cogs to the bot.
cogs_management.add_cogs(bot, './cogs')

# Runs bot.
bot.run(config.TOKEN)
