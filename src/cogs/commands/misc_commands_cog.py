from discord.ext import commands
from datetime import datetime
import discord

class MiscCommands(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    """Basic ping command to get bot's latency."""
    @commands.command()
    async def ping(self, ctx):
        latency = round(self.bot.latency * 1000)

        embed = discord.Embed(title=f':ping_pong: {latency}ms.', color=ctx.author.color)

        embed.set_author(name=ctx.author.display_name, icon_url=ctx.author.avatar_url)

        embed.timestamp = datetime.utcnow()
        embed.set_footer(text='OpenCDD - Flowerybee#2006\'s fork.')

        await ctx.reply(embed=embed)

def setup(bot):
    bot.add_cog(MiscCommands(bot))
