import discord
import markovify

token = "INSERT_TOKEN_HERE"

print("Creating client")
client = discord.Client()

model = markovify.Text("Hello, I am a bot.", state_size=1)
freq = 1

@client.event
async def on_message(message):
	global model
	global freq
	if message.author == client.user:
		return
	print("Recieved message:", message.content)
	message_id = int(message.id)
	print("Message id:", message_id)
	# we do not want the bot to reply to itself
	if message.content.startswith("!nonsensefreq"):
		freq = max(0, int(message.content.split()[1]))
		return
	model__ = markovify.Text(message.content, state_size=1)
	model = markovify.combine(models=[model, model__])
	print("Frequency:", freq)
	if message_id % freq == 0:
		sentence = model.make_short_sentence(2000, 1)
		if sentence is not None:
			print("Sending message:", sentence)
			await client.send_message(message.channel, sentence)

@client.event
async def on_ready():
	print("Logged in as")
	print(client.user.name)
	print(client.user.id)
	print('------')

print("Running client")
client.run(token)
