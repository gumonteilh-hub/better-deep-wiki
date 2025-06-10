Les équipes sont sauvegardées en appelant la fonction `handleSaveTeam` avec un objet contenant l'ID, le nom et les personnages de l'équipe. Cette fonction est appelée dans le composant `TeamSelection` lorsque l'utilisateur clique sur le bouton "Save".

Voici les sources qui permettent de répondre :

- Dans `TeamSelection.tsx`, la fonction `saveCurrentTeam` est définie pour gérer la validation et l'appel de `handleSaveTeam` :
  ```typescript
  function saveCurrentTeam(): void {
      let errorMessages: string[] = [];
      if (teamName == "") {
          errorMessages = [...errorMessages, "you need to give a name to your team"]
      }
      if (team.leaders.length !== 2 || team.characters.length !== 5) {
          errorMessages = [...errorMessages, "you must select a team of 7 characters"]
      }
      if (errorMessages.length > 0) {
          errorMessages.forEach(msg => {
              console.log(msg)
              toast.error(msg, { role: 'error' });
          })
          return;
      }
      handleSaveTeam({
          id: uuidv4(),
          name: teamName,
          characters: [...team.leaders, ...team.characters],
      });
      setTeam({ leaders: [], characters: [] });
      setTeamName("");
  }
  ```

- Le bouton "Save" appelle `saveCurrentTeam` lorsqu'il est cliqué :
  ```typescript
  <button className="saveButton" onClick={saveCurrentTeam}>
      Save
  </button>
  ```

- La structure de l'équipe à sauvegarder est définie dans `team.ts` :
  ```typescript
  export interface ITeamWithName {
      id: string;
      name: string;
      characters: Card[]
  }
  ```

Ces informations montrent clairement comment les équipes sont sauvegardées dans l'application.