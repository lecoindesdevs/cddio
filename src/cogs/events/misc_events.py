from discord.ext import commands

class MiscEvents(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    """Basic on_ready event which runs when bot is set."""
    @commands.Cog.listener()
    async def on_ready(self):
        print(f'I\'ve logged in as {self.bot.user}.')

def setup(bot):
    bot.add_cog(MiscEvents(bot))
