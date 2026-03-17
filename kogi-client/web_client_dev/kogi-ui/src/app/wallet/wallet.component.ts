import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-wallet',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './wallet.component.html',
  styleUrl: './wallet.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class WalletComponent {}
