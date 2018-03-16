import discord
import markovify

token = "INSERT_TOKEN_HERE"

print("Creating client")
client = discord.Client()

init_model = markovify.Text("Hello, I am a bot.", state_size=1)
model = init_model
freq = 1
max_chars = 140
channel = client.get_channel("381903453297049604")
if channel is None:
	print("ERROR: Default channel does not exist")

@client.event
async def on_message(message):
	global model
	global freq
	global max_chars
	global channel
	if message.author == client.user:
		return
	print("Recieved message:", message.content)
	message_id = int(message.id)
	print("Message id:", message_id)
	# we do not want the bot to reply to itself
	if message.content.startswith("!nonsense set freq"):
		freq = max(0, int(message.content.split()[3]))
		return
	if message.content.startswith("!nonsense get freq"):
		await client.send_message(channel, str(freq))
		return
	if message.content.startswith("!nonsense reset"):
		model = init_model
		return
	if message.content.startswith("!nonsense set maxchars"):
		max_chars = max(0, min(2000, int(message.content.split()[3])))
		return
	if message.content.startswith("!nonsense get maxchars"):
		await client.send_message(channel, str(max_chars))
		return
	if message.content.startswith("!nonsense set channel"):
		channel__ = client.get_channel(message.content.split()[3])
		if channel__ is None:
			await client.send_message(channel, "ERROR: Invalid channel")
		else:
			channel = channel__
		return
	model__ = markovify.Text(message.content, state_size=1)
	model = markovify.combine(models=[model, model__])
	print("Frequency:", freq)
	if message_id % freq == 0:
		sentence = model.make_short_sentence(max_chars, 1)
		if sentence is not None:
			print("Sending message:", sentence)
			await client.send_message(channel, sentence)

@client.event
async def on_ready():
	print("Logged in as")
	print(client.user.name)
	print(client.user.id)
	print('------')

print("Running client")
client.run(token)
