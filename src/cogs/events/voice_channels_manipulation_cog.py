from discord.ext import commands
import config

class VoiceChannelsManipulation(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    @commands.Cog.listener()
    async def on_voice_state_update(self, member, before, after):
        if after.channel != None and after.channel.id == config.CHANNELS['create_voice_channel']:
            new_channel_name = f'{member.name}#{member.discriminator}'

            for channel in after.channel.category.channels:
                if channel.name == new_channel_name:
                    return

            new_channel = await after.channel.category.create_voice_channel(new_channel_name)
            await member.move_to(new_channel)

        elif before.channel.category_id == config.CHANNELS['voice_category'] and not before.channel.id in [config.CHANNELS['create_voice_channel'], config.CHANNELS['afk_channel']]:
            if len(before.channel.members) < 1:
                await before.channel.delete()

def setup(bot):
    bot.add_cog(VoiceChannelsManipulation(bot))