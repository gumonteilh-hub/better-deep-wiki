Feuille de route DeepWiki-Rust
1. Initialisation du projet
 Créer le repo Rust (avec Cargo, structure modulaire dès le départ)

 Rédiger le README d’intention/philosophie (explicite sur les choix d’architecture, le “no magic”, la cible OpenAI/Mistral, etc.)

 Mettre en place les outils de formatage/linting/tests (rustfmt, clippy, CI minimale)

2. Traitement de base du repo
 Entrée CLI : chemin d’un dossier local contenant un projet code déjà cloné

 Scan récursif du repo, exclusion configurable de certains dossiers/fichiers (patterns .gitignore-like)

 Découpage du code en chunks (par fichier, classe, fonction, ou lignes) – à tester et ajuster selon les besoins RAG

 Gestion des erreurs robustes sur les accès fichiers, dossiers inexistants, encodages, etc.

3. Intégration API d’embedding
 Interface d’appel générique à une API OpenAI-compatible (clé/URL/API model en config simple)

 Implémentation concrète : OpenAI embeddings

 Implémentation concrète : MistralAI embeddings (compatible OpenAI)

 Gestion de la rate-limit, du batching et du retry

4. Stockage et recherche vectorielle
 Choix d’une solution simple : SQLite (local), ou support optionnel d’un service vectoriel type Qdrant/Pinecone (mais en second plan)

 Stockage local des embeddings, mapping chunk<->embedding

 Recherche de similarité (cosinus, top-k)

 API interne/CLI pour interroger la base vectorielle

5. Composant RAG (Retrieval-Augmented Generation)
 Appel du LLM (OpenAI/Mistral) pour générer la réponse à une question donnée, en injectant les chunks les plus proches

 Gestion context window (limiter la taille de la prompt contextuelle)

 CLI simple ou API HTTP pour tester la chaîne RAG (pose une question, reçois la réponse augmentée)

6. Interface utilisateur minimale
 (Optionnel) Petite API web (avec e.g. axum ou warp), ou simple CLI interactive pour usage direct

 Interface web simple (si besoin) : champ de question, affichage résultat, log des requêtes

7. Qualité, DX et doc
 Ajout de logs, gestion d’erreurs et reporting clairs

 Documentation claire (README, utilisation, limites, choix techniques)

 Exemples d’utilisation

 Suite de tests unitaires (scénarios de chunking, embedding, recherche)

 Scripts de build et déploiement (release binaire, Docker si besoin)

8. Extensions (si le projet reste simple)
 Prise en charge de plusieurs repos/index

 Support optionnel d’un stockage cloud (S3, GCS)

 Export/Import de la base vectorielle

NB :
Chacune de ces étapes peut être découpée en issues GitHub et traitée itérativement. Tu peux pousser du code utilisable après chaque milestone, pour valider le fonctionnement en conditions réelles rapidement.

