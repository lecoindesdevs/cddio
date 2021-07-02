from discord.ext import commands

class MiscCommands(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    """Basic ping command to get bot's latency."""
    @commands.command()
    async def ping(self, ctx):
        await ctx.reply(f':ping_pong: {round(self.bot.latency * 1000)}ms.')

def setup(bot):
    bot.add_cog(MiscCommands(bot))
