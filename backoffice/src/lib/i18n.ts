import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

export type Locale = 'en' | 'pt';

// Persist language preference in localStorage
const stored = browser ? (localStorage.getItem('mailmate-lang') as Locale) : null;
export const locale = writable<Locale>(stored || 'en');

// Sync to localStorage
if (browser) {
	locale.subscribe((val) => localStorage.setItem('mailmate-lang', val));
}

// ─── Translation Dictionaries ────────────────────────────────────────────────

const translations: Record<string, Record<Locale, string>> = {
	// ── Sidebar / Navigation ────────────────────────────────────────────────
	'nav.dashboard': { en: 'Dashboard', pt: 'Painel' },
	'nav.emails': { en: 'Emails', pt: 'E-mails' },
	'nav.users': { en: 'Users', pt: 'Utilizadores' },
	'nav.roles': { en: 'Roles', pt: 'Funções' },
	'nav.settings': { en: 'Settings', pt: 'Definições' },
	'nav.knowledge': { en: 'Knowledge Base', pt: 'Base de Conhecimento' },
	'nav.audit': { en: 'Audit Logs', pt: 'Registos de Auditoria' },
	'nav.signout': { en: 'Sign Out', pt: 'Terminar Sessão' },
	'nav.signin': { en: 'Sign In', pt: 'Iniciar Sessão' },

	// ── Header ──────────────────────────────────────────────────────────────
	'header.connected': { en: 'Connected', pt: 'Conectado' },
	'header.admin': { en: 'Admin', pt: 'Administrador' },
	'header.user': { en: 'User', pt: 'Utilizador' },

	// ── Dashboard ────────────────────────────────────────────────────────────
	'dash.title': { en: 'Dashboard', pt: 'Painel' },
	'dash.subtitle_admin': { en: 'Administration overview of your AI email assistant.', pt: 'Visão geral da administração do seu assistente de e-mail com IA.' },
	'dash.subtitle_user': { en: 'Overview of your AI email assistant.', pt: 'Visão geral do seu assistente de e-mail com IA.' },
	'dash.emails_generated': { en: 'Emails generated', pt: 'E-mails gerados' },
	'dash.users': { en: 'Users', pt: 'Utilizadores' },
	'dash.roles': { en: 'Roles', pt: 'Funções' },
	'dash.llm_model': { en: 'LLM model', pt: 'Modelo LLM' },
	'dash.not_set': { en: 'Not set', pt: 'Não definido' },
	'dash.llm_config': { en: 'LLM Configuration', pt: 'Configuração LLM' },
	'dash.model': { en: 'Model', pt: 'Modelo' },
	'dash.base_url': { en: 'Base URL', pt: 'URL Base' },
	'dash.api_key': { en: 'API Key', pt: 'Chave API' },
	'dash.configured': { en: 'Configured', pt: 'Configurada' },
	'dash.missing': { en: 'Missing', pt: 'Ausente' },
	'dash.embedding_model': { en: 'Embedding Model', pt: 'Modelo de Embeddings' },
	'dash.rag_health': { en: 'RAG Context Health', pt: 'Saúde do Contexto RAG' },
	'dash.context_depth': { en: 'Context Depth', pt: 'Profundidade do Contexto' },
	'dash.similarity': { en: 'Similarity', pt: 'Similaridade' },
	'dash.accuracy': { en: 'Accuracy', pt: 'Precisão' },
	'dash.telemetry': { en: 'AI Execution Telemetry', pt: 'Telemetria de Execução IA' },
	'dash.context_rate': { en: 'Context Retrieval Rate', pt: 'Taxa de Recuperação de Contexto' },
	'dash.tokens_processed': { en: 'Tokens Processed', pt: 'Tokens Processados' },
	'dash.avg_similarity': { en: 'Avg. Similarity', pt: 'Similaridade Média' },
	'dash.knowledge_matches': { en: 'Knowledge Matches', pt: 'Correspondências de Conhecimento' },
	'dash.recent': { en: 'Recent Generations', pt: 'Gerações Recentes' },
	'dash.view_all': { en: 'View all emails', pt: 'Ver todos os e-mails' },
	'dash.no_emails': { en: 'No emails generated yet.', pt: 'Nenhum e-mail gerado ainda.' },

	// ── Emails Page ──────────────────────────────────────────────────────────
	'emails.title': { en: 'Generated Emails', pt: 'E-mails Gerados' },
	'emails.subtitle': { en: 'View and manage AI-generated email replies.', pt: 'Ver e gerir respostas de e-mail geradas por IA.' },
	'emails.search': { en: 'Search emails...', pt: 'Pesquisar e-mails...' },
	'emails.generate': { en: 'Generate Reply', pt: 'Gerar Resposta' },
	'emails.original': { en: 'Original Email', pt: 'E-mail Original' },
	'emails.generated': { en: 'Generated Reply', pt: 'Resposta Gerada' },
	'emails.copy': { en: 'Copy Reply', pt: 'Copiar Resposta' },
	'emails.copied': { en: 'Copied!', pt: 'Copiado!' },
	'emails.delete': { en: 'Delete', pt: 'Eliminar' },
	'emails.confirm_delete': { en: 'Confirm Delete?', pt: 'Confirmar Eliminação?' },
	'emails.deleting': { en: 'Deleting...', pt: 'A eliminar...' },
	'emails.no_emails': { en: 'No emails generated yet. Click "Generate Reply" to get started.', pt: 'Nenhum e-mail gerado ainda. Clique em "Gerar Resposta" para começar.' },
	'emails.prompt_label': { en: 'Paste the incoming email you want to reply to:', pt: 'Cole o e-mail recebido ao qual pretende responder:' },
	'emails.prompt_placeholder': { en: 'Paste the email content here...', pt: 'Cole o conteúdo do e-mail aqui...' },
	'emails.generating': { en: 'Generating...', pt: 'A gerar...' },
	'emails.cancel': { en: 'Cancel', pt: 'Cancelar' },
	'emails.previous': { en: 'Previous', pt: 'Anterior' },
	'emails.next': { en: 'Next', pt: 'Seguinte' },

	// ── Users Page ───────────────────────────────────────────────────────────
	'users.title': { en: 'User Management', pt: 'Gestão de Utilizadores' },
	'users.subtitle': { en: 'Create and manage user accounts.', pt: 'Criar e gerir contas de utilizadores.' },
	'users.add': { en: 'Add User', pt: 'Adicionar Utilizador' },
	'users.email': { en: 'Email', pt: 'E-mail' },
	'users.password': { en: 'Password', pt: 'Palavra-passe' },
	'users.role': { en: 'Role', pt: 'Função' },
	'users.created': { en: 'Created', pt: 'Criado' },
	'users.actions': { en: 'Actions', pt: 'Ações' },
	'users.no_users': { en: 'No users found.', pt: 'Nenhum utilizador encontrado.' },

	// ── Roles Page ───────────────────────────────────────────────────────────
	'roles.title': { en: 'Role Management', pt: 'Gestão de Funções' },
	'roles.subtitle': { en: 'Manage roles and permissions.', pt: 'Gerir funções e permissões.' },
	'roles.name': { en: 'Role Name', pt: 'Nome da Função' },
	'roles.add': { en: 'Add Role', pt: 'Adicionar Função' },
	'roles.no_roles': { en: 'No roles found.', pt: 'Nenhuma função encontrada.' },

	// ── Settings Page ────────────────────────────────────────────────────────
	'settings.title': { en: 'LLM Settings', pt: 'Definições LLM' },
	'settings.subtitle': { en: 'Configure the AI model provider and API key.', pt: 'Configure o fornecedor do modelo de IA e a chave API.' },
	'settings.base_url': { en: 'LLM Base URL', pt: 'URL Base do LLM' },
	'settings.model': { en: 'Model Name', pt: 'Nome do Modelo' },
	'settings.api_key': { en: 'API Key', pt: 'Chave API' },
	'settings.save': { en: 'Save Settings', pt: 'Guardar Definições' },
	'settings.verify': { en: 'Verify Connection', pt: 'Verificar Conexão' },
	'settings.saving': { en: 'Saving...', pt: 'A guardar...' },
	'settings.verifying': { en: 'Verifying...', pt: 'A verificar...' },

	// ── Knowledge Page ───────────────────────────────────────────────────────
	'knowledge.title': { en: 'Knowledge Base', pt: 'Base de Conhecimento' },
	'knowledge.subtitle': { en: 'Manage documents and Q&A pairs that power the RAG pipeline.', pt: 'Gerir documentos e pares Q&R que alimentam o pipeline RAG.' },
	'knowledge.qa_tab': { en: 'Q&A Pairs', pt: 'Pares Q&R' },
	'knowledge.docs_tab': { en: 'Documents', pt: 'Documentos' },
	'knowledge.add_qa': { en: 'Add Q&A Pair', pt: 'Adicionar Par Q&R' },
	'knowledge.qa_description': { en: 'Add a question and its ideal answer. The question is embedded so that when similar incoming emails arrive, the answer is used as RAG context.', pt: 'Adicione uma pergunta e a resposta ideal. A pergunta é embedida para que quando e-mails semelhantes cheguem, a resposta seja usada como contexto RAG.' },
	'knowledge.question': { en: 'Question', pt: 'Pergunta' },
	'knowledge.answer': { en: 'Answer', pt: 'Resposta' },
	'knowledge.add_entry': { en: 'Add Entry', pt: 'Adicionar Entrada' },
	'knowledge.embedding': { en: 'Embedding...', pt: 'A criar embedding...' },
	'knowledge.existing_qa': { en: 'Existing Q&A Pairs', pt: 'Pares Q&R Existentes' },
	'knowledge.no_qa': { en: 'No Q&A pairs yet. Add one above to enrich the RAG context.', pt: 'Sem pares Q&R. Adicione um acima para enriquecer o contexto RAG.' },
	'knowledge.upload_doc': { en: 'Upload Document', pt: 'Carregar Documento' },
	'knowledge.upload_description': { en: 'Upload a document (.txt, .md, .pdf). It will be chunked and each chunk embedded for RAG retrieval.', pt: 'Carregue um documento (.txt, .md, .pdf). Será dividido em partes e cada parte será embedida para recuperação RAG.' },
	'knowledge.click_upload': { en: 'Click to upload', pt: 'Clique para carregar' },
	'knowledge.or_drag': { en: 'or drag and drop', pt: 'ou arraste e largue' },
	'knowledge.processing': { en: 'Extracting, chunking & embedding...', pt: 'A extrair, dividir e criar embeddings...' },
	'knowledge.uploaded_docs': { en: 'Uploaded Documents', pt: 'Documentos Carregados' },
	'knowledge.no_docs': { en: 'No documents uploaded yet. Drag a file above to get started.', pt: 'Nenhum documento carregado. Arraste um ficheiro acima para começar.' },

	// ── Audit Logs Page ──────────────────────────────────────────────────────
	'audit.title': { en: 'Audit Logs', pt: 'Registos de Auditoria' },
	'audit.subtitle': { en: 'System-wide record of administrative actions.', pt: 'Registo global de ações administrativas.' },
	'audit.search': { en: 'Filter by action or user ID...', pt: 'Filtrar por ação ou ID de utilizador...' },
	'audit.timestamp': { en: 'Timestamp', pt: 'Data/Hora' },
	'audit.actor': { en: 'Actor User ID', pt: 'ID do Utilizador' },
	'audit.action': { en: 'Action', pt: 'Ação' },
	'audit.details': { en: 'Details', pt: 'Detalhes' },
	'audit.view_json': { en: 'View JSON', pt: 'Ver JSON' },
	'audit.payload': { en: 'Payload Metadata', pt: 'Metadados do Payload' },
	'audit.no_logs': { en: 'No audit logs recorded yet.', pt: 'Nenhum registo de auditoria ainda.' },
	'audit.no_match': { en: 'No matching logs found.', pt: 'Nenhum registo correspondente encontrado.' },
	'audit.showing': { en: 'Showing', pt: 'A mostrar' },
	'audit.of': { en: 'of', pt: 'de' },
	'audit.events': { en: 'events', pt: 'eventos' },

	// ── Login Page ───────────────────────────────────────────────────────────
	'login.title': { en: 'Sign in to MailMate', pt: 'Iniciar sessão no MailMate' },
	'login.email': { en: 'Email address', pt: 'Endereço de e-mail' },
	'login.password': { en: 'Password', pt: 'Palavra-passe' },
	'login.submit': { en: 'Sign In', pt: 'Entrar' },
	'login.no_account': { en: "Don't have an account?", pt: 'Não tem conta?' },
	'login.register': { en: 'Create one', pt: 'Criar uma' },
	'login.or': { en: 'or continue with', pt: 'ou continue com' },
	'login.google': { en: 'Sign in with Google', pt: 'Entrar com o Google' },

	// ── Common ───────────────────────────────────────────────────────────────
	'common.previous': { en: 'Previous', pt: 'Anterior' },
	'common.next': { en: 'Next', pt: 'Seguinte' },
	'common.showing': { en: 'Showing', pt: 'A mostrar' },
	'common.to': { en: 'to', pt: 'até' },
	'common.of': { en: 'of', pt: 'de' },
	'common.delete': { en: 'Delete', pt: 'Eliminar' },
	'common.save': { en: 'Save', pt: 'Guardar' },
	'common.cancel': { en: 'Cancel', pt: 'Cancelar' },
	'common.loading': { en: 'Loading...', pt: 'A carregar...' },
	'common.system': { en: 'System', pt: 'Sistema' },
	'common.no_metadata': { en: 'No metadata', pt: 'Sem metadados' },
};

// ─── Translation Function ────────────────────────────────────────────────────

export function t(key: string, currentLocale: Locale): string {
	return translations[key]?.[currentLocale] || key;
}

// Reactive derived store for convenience
export const currentT = derived(locale, ($locale) => {
	return (key: string) => t(key, $locale);
});
