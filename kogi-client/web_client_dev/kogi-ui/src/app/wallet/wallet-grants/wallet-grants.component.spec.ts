import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletGrantsComponent } from './wallet-grants.component';

describe('WalletGrantsComponent', () => {
  let component: WalletGrantsComponent;
  let fixture: ComponentFixture<WalletGrantsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletGrantsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletGrantsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
