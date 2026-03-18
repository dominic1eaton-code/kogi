import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-spaces-network-linktree-editor',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './network-linktree-editor.component.html',
  styleUrl: './network-linktree-editor.component.css',
  host: {
    class: 'block w-full'
  }
})
export class SpacesNetworkLinktreeEditorComponent {}
