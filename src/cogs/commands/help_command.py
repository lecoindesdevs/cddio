from discord.ext.commands import MinimalHelpCommand
from discord import Embed

class HelpCommand(MinimalHelpCommand):
    async def send_pages(self):
        dest = self.get_destination()

        embed = Embed(description='')

        for page in self.paginator.pages:
            embed.description += page

        await dest.send(embed=embed)
