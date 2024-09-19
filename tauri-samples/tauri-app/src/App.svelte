<script>
	import Modal from './Modal.svelte';
	import AddPersonForm from './AddPersonForm.svelte';
	const invoke = window.__TAURI__.invoke

	let people = [
    { name: 'yoshi', beltColour: 'black', age: 25, id: 1 },
    { name: 'mario', beltColour: 'orange', age: 45, id: 2 },
    { name: 'luigi', beltColour: 'brown', age: 35, id: 3 }
  ];

	let showModal = false;
	const toggleModal = () => {
		showModal = !showModal;
	}

  const handleClick = (e, id) => {
    people = people.filter(person => person.id != id);
    console.log(e);
  };

  const addPerson = (e) => {
	const person = e.detail;
	people = [person, ...people]
	toggleModal();
  }

</script>

<Modal {showModal} on:click={toggleModal}>
	<AddPersonForm on:addPerson={addPerson}/>
</Modal>
<main>
	<button on:click ={toggleModal}>Open Modal</button>
	<button on:click ={() => {invoke('greet',{name: 'Abu'})}}>Call the Rust</button>
  	{#each people as person (person.id)}
		<div>
		<h4>{person.name}</h4>
		{#if person.beltColour === 'brown'}
			<p><strong>MASTER NINJA</strong></p>
		{/if}
		<p>{person.age} years old, {person.beltColour} belt.</p>
		<button on:click={(e) => handleClick(e, person.id)}>delete</button>
		</div>
    {:else}
    	<p>There are no people to show...</p>
  	{/each}
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>